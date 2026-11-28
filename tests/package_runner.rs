//! FR-009 and NFR-003 `run-package` over a stub `producer-execute` (TC-015).
//!
//! Every contract package is built at test time (`common`); nothing is committed. The stub is a
//! `/bin/sh` script that records each request it receives and answers with scripted result lines.

#![cfg(target_os = "linux")]

mod common;

use std::{
    ffi::OsString,
    fs,
    io::Write,
    os::unix::{
        ffi::OsStringExt,
        fs::{symlink, PermissionsExt},
    },
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

use common::{
    cycle_document, id, integer_cycle_document, labels, not_chain, option_value, product_document,
    record_field, with_identity, Document, Sized, TempDir, AND, ENUM_EQ, NOT,
};
use filament_analyze::{
    lower_analysis_request, translate_analysis_claims, validate_report_document,
    DifferentialDisposition,
};
use filament_contract_model::{
    CheckedPackageReadLimits, ContractPackageReadResult, ContractPackageWireV1,
};
use ix_trace_rs::trace;
use serde_json::{json, Value};

const BINARY: &str = env!("CARGO_BIN_EXE_filament-analyze");
const VERSION_Z3: &str = "Z3 version 4.13.0 - 64 bit\n";
const VERSION_CVC5: &str = "This is cvc5 version 1.1.2\n";
const RESULT_PROTOCOL: &str = "engineering-assurance.producer-execution-result/v1";

/// A stub that logs the request, the mode of the capability root and the staged query, and
/// answers from `data/{engine}.{kind}` where kind is `version` or `query`.
fn stub_script(data: &Path) -> String {
    let data = data.display();
    format!(
        r#"#!/bin/sh
D='{data}'
req=
while IFS= read -r l || [ -n "$l" ]; do req="$req$l"; done
printf '%s\n' "$req" >> "$D/requests.log"
case "$req" in *'"stdin":{{"kind":"null"}}'*) kind=version;; *) kind=query;; esac
case "$req" in *'"producer":{{"name":"z3"'*) eng=z3;; *) eng=cvc5;; esac
root=${{req#*'"capabilityRoot":"'}}
root=${{root%%'"'*}}
stat -c %a "$root" >> "$D/modes.log"
if [ "$kind" = query ]; then cp "$root/query.smt2" "$D/$eng.staged"; fi
f="$D/$eng.$kind"
if [ -f "$f.signal" ]; then kill -KILL $$; fi
if [ -f "$D/undeletable" ] && [ ! -e "$root/locked" ]; then
  mkdir -p "$root/locked"; : > "$root/locked/entry"; chmod 000 "$root/locked"
fi
if [ -f "$f.flood" ]; then yes aaaaaaaaaaaaaaa | head -c 70000000; exit 0; fi
if [ -f "$f" ]; then
  while IFS= read -r l || [ -n "$l" ]; do printf '%s\n' "$l"; done < "$f"
fi
if [ -f "$f.exit" ]; then read -r c < "$f.exit"; exit "$c"; fi
exit 0
"#
    )
}

fn process(exit: i64, stdout: &str, stderr: &str) -> Value {
    json!({
        "terminalStatus": {"kind": "exit_code", "value": exit},
        "stdout": {"bytes": stdout.as_bytes(), "truncated": false},
        "stderr": {"bytes": stderr.as_bytes(), "truncated": false},
    })
}

fn line(state: &Value, process: Option<Value>) -> String {
    let document = json!({
        "protocol": RESULT_PROTOCOL,
        "producer": {"name": "z3", "version": "stub", "executable": "/stub"},
        "cancellationEvent": null,
        "timing": {"preflightNanos": 1, "executionNanos": 3_000_000, "totalNanos": 4_000_000},
        "observedHost": null,
        "process": process,
        "artifacts": [],
        "state": state,
    });
    serde_json::to_string(&document).expect("result line")
}

fn completed(exit: i64, stdout: &str, stderr: &str) -> String {
    line(
        &json!({"kind": "completed", "observation": {"exitCode": exit}}),
        Some(process(exit, stdout, stderr)),
    )
}

fn state_only(state: Value) -> String {
    line(&state, None)
}

/// A package, and the model response that satisfies its query.
struct Built {
    bytes: Vec<u8>,
    model: String,
}

fn model_for(bytes: &[u8]) -> String {
    let package = match ContractPackageWireV1::read(
        bytes,
        CheckedPackageReadLimits::bounded(),
        &Default::default(),
    ) {
        ContractPackageReadResult::Admitted(package) => *package,
        other => panic!("fixture refused: {other:?}"),
    };
    let translation = translate_analysis_claims(&package).expect("translation");
    let query = lower_analysis_request(translation.request()).expect("query");
    let definitions = query
        .variables()
        .iter()
        .map(|variable| format!("(define-fun {} () Bool true)", variable.symbol))
        .collect::<Vec<_>>()
        .join(" ");
    format!("sat\n(model {definitions})\n")
}

/// Two parameters, each asserted: every assignment of true satisfies them.
fn consistent() -> Built {
    let mut document = Document::new();
    let p = document.parameter();
    let q = document.parameter();
    let first = document.claim(&p);
    let second = document.claim(&q);
    let bytes = document.bytes(&[&first, &second]);
    Built {
        model: model_for(&bytes),
        bytes,
    }
}

/// A parameter and its negation, both asserted.
fn contradictory() -> Built {
    let mut document = Document::new();
    let p = document.parameter();
    let not_p = document.op(NOT, &[&p]);
    let first = document.claim(&p);
    let second = document.claim(&not_p);
    let bytes = document.bytes(&[&first, &second]);
    Built {
        model: model_for(&bytes),
        bytes,
    }
}

struct Run {
    code: Option<i32>,
    stdout: String,
    stderr: String,
}

struct Fixture {
    dir: TempDir,
    package: PathBuf,
}

impl Fixture {
    fn new(package: &[u8]) -> Self {
        let dir = TempDir::new("run-package");
        for name in ["pkg", "out", "stub", "stub/data", "solvers"] {
            fs::create_dir(dir.path().join(name)).expect("directory");
        }
        let package_path = dir.path().join("pkg/package.json");
        fs::write(&package_path, package).expect("package");
        let producer = dir.path().join("stub/producer");
        fs::write(&producer, stub_script(&dir.path().join("stub/data"))).expect("stub");
        fs::set_permissions(&producer, fs::Permissions::from_mode(0o700)).expect("executable");
        let fixture = Self {
            dir,
            package: package_path,
        };
        fixture.answer_all(&consistent_answers());
        fixture
    }

    fn path(&self, relative: &str) -> PathBuf {
        self.dir.path().join(relative)
    }

    fn producer(&self) -> PathBuf {
        self.path("stub/producer")
    }

    fn z3(&self) -> PathBuf {
        self.path("solvers/z3")
    }

    fn cvc5(&self) -> PathBuf {
        self.path("solvers/cvc5")
    }

    fn output(&self) -> PathBuf {
        self.path("out/report.json")
    }

    fn answer(&self, engine: &str, kind: &str, body: &str) {
        fs::write(self.path(&format!("stub/data/{engine}.{kind}")), body).expect("answer");
    }

    fn answer_all(&self, answers: &[(&str, &str, String)]) {
        for (engine, kind, body) in answers {
            self.answer(engine, kind, body);
        }
    }

    fn exit_after(&self, engine: &str, kind: &str, code: i32) {
        fs::write(
            self.path(&format!("stub/data/{engine}.{kind}.exit")),
            code.to_string(),
        )
        .expect("exit");
    }

    fn signal(&self, engine: &str, kind: &str) {
        fs::write(self.path(&format!("stub/data/{engine}.{kind}.signal")), "").expect("signal");
    }

    /// Both engines answer their version and, for the queries, `z3` and `cvc5`.
    fn both(&self, z3: &str, cvc5: &str) {
        self.answer_all(&answers(z3, cvc5));
    }

    /// Change the `producer-execute` execution time every scripted answer reports.
    fn retime(&self, from_nanos: u64, to_nanos: u64) {
        for entry in fs::read_dir(self.path("stub/data")).expect("data") {
            let path = entry.expect("entry").path();
            let Ok(text) = fs::read_to_string(&path) else {
                continue;
            };
            let from = format!("\"executionNanos\":{from_nanos}");
            if text.contains(&from) {
                let to = format!("\"executionNanos\":{to_nanos}");
                fs::write(&path, text.replace(&from, &to)).expect("retime");
            }
        }
    }

    fn arguments(&self) -> Vec<OsString> {
        let pairs: [(&str, &Path); 5] = [
            ("--package", &self.package),
            ("--output", &self.output()),
            ("--z3", &self.z3()),
            ("--cvc5", &self.cvc5()),
            ("--producer-execute", &self.producer()),
        ];
        let mut arguments = vec![OsString::from("run-package")];
        for (name, value) in pairs {
            arguments.push(name.into());
            arguments.push(value.into());
        }
        arguments
    }

    fn command(&self, arguments: &[OsString]) -> Command {
        let mut command = Command::new(BINARY);
        command.args(arguments).stdin(Stdio::null());
        command
    }

    fn run(&self) -> Run {
        self.run_with(&self.arguments())
    }

    fn run_with(&self, arguments: &[OsString]) -> Run {
        finish(self.command(arguments))
    }

    /// Replace the value of `option` in the standard arguments.
    fn with(&self, option: &str, value: impl Into<OsString>) -> Vec<OsString> {
        let mut arguments = self.arguments();
        let position = arguments
            .iter()
            .position(|argument| argument == option)
            .expect("option");
        arguments[position + 1] = value.into();
        arguments
    }

    /// The standard arguments with the real executable and the real solvers.
    fn with_real_tools(&self, executable: PathBuf, z3: PathBuf, cvc5: PathBuf) -> Vec<OsString> {
        let mut arguments = self.with("--producer-execute", executable);
        for (option, value) in [("--z3", z3), ("--cvc5", cvc5)] {
            let position = arguments.iter().position(|a| a == option).expect("option");
            arguments[position + 1] = value.into();
        }
        arguments
    }

    fn without(&self, option: &str) -> Vec<OsString> {
        let mut arguments = self.arguments();
        let position = arguments
            .iter()
            .position(|argument| argument == option)
            .expect("option");
        arguments.drain(position..=position + 1);
        arguments
    }

    fn requests(&self) -> Vec<Value> {
        fs::read_to_string(self.path("stub/data/requests.log"))
            .unwrap_or_default()
            .lines()
            .map(|text| serde_json::from_str(text).expect("logged request"))
            .collect()
    }

    fn report(&self) -> Value {
        serde_json::from_slice(&fs::read(self.output()).expect("report")).expect("report json")
    }

    fn assert_no_report(&self) {
        assert!(!self.output().exists(), "a report was written");
        self.assert_no_leftovers();
    }

    /// The only entries of the output directory are the published report.
    fn assert_no_leftovers(&self) {
        let names = directory_names(&self.path("out"));
        assert!(
            names.iter().all(|name| name == "report.json"),
            "unexpected entries {names:?}"
        );
    }
}

struct SelectionFixture {
    fixture: Fixture,
    first_id: String,
    first_claim: (u64, u64),
    first_parameter: (u64, u64),
    second_id: String,
    second_claim: (u64, u64),
    second_parameter: (u64, u64),
}

fn selection_fixture() -> SelectionFixture {
    let fixture = Fixture::new(b"pending package");
    let mut document = Document::new();
    document.source_identity = "rules/a.qsl".to_owned();
    let first_parameter = document.parameter();
    let first_parameter_region = document.region(&first_parameter);
    let first_claim = document.claim(&first_parameter);
    let first_claim_region = document.region(&first_claim);
    let second_parameter = document.parameter();
    let second_parameter_region = document.region(&second_parameter);
    let second_claim = document.claim(&second_parameter);
    let second_claim_region = document.region(&second_claim);
    let package = document.bytes(&[&first_claim, &second_claim]);
    fs::write(&fixture.package, package).expect("write package");
    SelectionFixture {
        fixture,
        first_id: "claim-0".to_owned(),
        first_claim: first_claim_region,
        first_parameter: first_parameter_region,
        second_id: "claim-1".to_owned(),
        second_claim: second_claim_region,
        second_parameter: second_parameter_region,
    }
}

fn selection_file(fixture: &Fixture, spans: Value) -> PathBuf {
    let path = fixture.path("selection.json");
    fs::write(&path, selection_bytes(spans)).expect("selection file");
    path
}

fn selection_bytes(spans: Value) -> Vec<u8> {
    serde_json::to_vec(&json!({
        "protocol": "filament-analyze.claim-selection/v1",
        "spans": spans,
    }))
    .expect("selection json")
}

fn selection_span(start: u64, end: u64) -> Value {
    json!({
        "authority": "agent-ix",
        "identity": "rules/a.qsl",
        "start": start,
        "end": end,
    })
}

fn directory_names(path: &Path) -> Vec<String> {
    let mut names = fs::read_dir(path)
        .expect("directory")
        .map(|entry| {
            entry
                .expect("entry")
                .file_name()
                .to_string_lossy()
                .into_owned()
        })
        .collect::<Vec<_>>();
    names.sort();
    names
}

fn finish(mut command: Command) -> Run {
    let output = command.output().expect("run filament-analyze");
    Run {
        code: output.status.code(),
        stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
    }
}

/// Set every `elapsedMs` field to `0`: the one field FR-009 excludes from determinism.
fn zero_elapsed(value: &mut Value) {
    match value {
        Value::Object(map) => {
            for (key, entry) in map.iter_mut() {
                if key == "elapsedMs" {
                    *entry = json!(0);
                } else {
                    zero_elapsed(entry);
                }
            }
        }
        Value::Array(items) => items.iter_mut().for_each(zero_elapsed),
        _ => {}
    }
}

/// The report bytes parsed with every `elapsedMs` set to `0`.
fn normalised(bytes: &[u8]) -> Value {
    let mut value: Value = serde_json::from_slice(bytes).expect("report json");
    zero_elapsed(&mut value);
    value
}

fn answers<'a>(z3: &str, cvc5: &str) -> Vec<(&'a str, &'a str, String)> {
    vec![
        ("z3", "version", completed(0, VERSION_Z3, "")),
        ("z3", "query", completed(0, z3, "")),
        ("cvc5", "version", completed(0, VERSION_CVC5, "")),
        ("cvc5", "query", completed(0, cvc5, "")),
    ]
}

fn consistent_answers<'a>() -> Vec<(&'a str, &'a str, String)> {
    answers("unknown\n", "unknown\n")
}

fn outcomes(report: &Value) -> (String, String) {
    let outcome = |index: usize| {
        report["engines"][index]["outcome"]
            .as_str()
            .expect("outcome")
            .to_owned()
    };
    (outcome(0), outcome(1))
}

#[test]
fn explicit_consistency_is_byte_identical_to_the_default_request() {
    let built = consistent();
    let fixture = Fixture::new(&built.bytes);
    fixture.both("unknown\n", "unknown\n");

    let default = fixture.run();
    assert_eq!(default.code, Some(3), "{}", default.stderr);
    let default_bytes = fs::read(fixture.output()).expect("default report");
    fs::remove_file(fixture.output()).expect("remove default report");

    let mut explicit_arguments = fixture.arguments();
    explicit_arguments.extend(["--request".into(), "consistency".into()]);
    let explicit = fixture.run_with(&explicit_arguments);
    assert_eq!(explicit.code, Some(3), "{}", explicit.stderr);
    let explicit_bytes = fs::read(fixture.output()).expect("explicit report");
    assert_eq!(explicit_bytes, default_bytes);
    assert!(fixture.report().get("selection").is_none());
}

#[trace("FR-009-AC-30", "FR-009-AC-37")]
#[test]
fn an_unmatched_exact_identity_span_names_the_span_and_writes_no_report() {
    let selection = selection_fixture();
    let variants = [
        (
            "unmatched region",
            "identity",
            json!("rules/a.qsl"),
            1000,
            1010,
        ),
        (
            "identity case",
            "identity",
            json!("Rules/a.qsl"),
            selection.first_claim.0,
            selection.first_claim.1,
        ),
        (
            "leading dot",
            "identity",
            json!("./rules/a.qsl"),
            selection.first_claim.0,
            selection.first_claim.1,
        ),
        (
            "doubled separator",
            "identity",
            json!("rules//a.qsl"),
            selection.first_claim.0,
            selection.first_claim.1,
        ),
        (
            "trailing separator",
            "identity",
            json!("rules/a.qsl/"),
            selection.first_claim.0,
            selection.first_claim.1,
        ),
        (
            "authority case",
            "authority",
            json!("Agent-ix"),
            selection.first_claim.0,
            selection.first_claim.1,
        ),
    ];
    for (label, field, value, start, end) in variants {
        let mut span = selection_span(start, end);
        span[field] = value;
        let selection_path = selection_file(&selection.fixture, json!([span]));
        let mut arguments = selection.fixture.arguments();
        arguments.extend(["--select".into(), selection_path.into()]);

        let run = selection.fixture.run_with(&arguments);
        assert_eq!(run.code, Some(2), "{label}: {}", run.stderr);
        assert!(
            run.stderr.contains("matches no claim"),
            "{label}: {}",
            run.stderr
        );
        selection.fixture.assert_no_report();
    }
}

#[trace("FR-009-AC-31")]
#[test]
fn malformed_and_oversized_selection_documents_are_usage_errors() {
    let selection = selection_fixture();
    let valid_span = selection_span(selection.first_claim.0, selection.first_claim.1);
    let mut malformed = vec![
        ("invalid JSON", b"{".to_vec()),
        (
            "unknown document field",
            br#"{"protocol":"filament-analyze.claim-selection/v1","spans":[],"extra":1}"#.to_vec(),
        ),
        ("missing protocol", br#"{"spans":[]}"#.to_vec()),
    ];
    let wrong_protocol = serde_json::to_vec(&json!({
        "protocol": "quire-analyze.claim-selection/v1",
        "spans": [valid_span.clone()],
    }))
    .expect("wrong protocol JSON");
    malformed.push(("wrong protocol", wrong_protocol));

    for (field, value) in [
        ("authority", json!("")),
        ("identity", json!("")),
        ("start", json!(-1)),
        ("start", json!(1.5)),
        ("start", json!("x")),
        ("end", json!(-1)),
        ("end", json!(1.5)),
        ("end", json!(null)),
        ("start", json!(selection.first_claim.1)),
    ] {
        let mut span = valid_span.clone();
        span[field] = value;
        malformed.push(("malformed span", selection_bytes(json!([span]))));
    }
    let missing_span_member = json!({
        "authority": "agent-ix",
        "identity": "rules/a.qsl",
        "start": 0,
    });
    malformed.push((
        "missing span member",
        selection_bytes(json!([missing_span_member])),
    ));
    for (label, start, end) in [
        (
            "start overflow",
            "18446744073709551616",
            "18446744073709551615",
        ),
        ("end overflow", "1", "18446744073709551616"),
    ] {
        malformed.push((
            label,
            format!(
                r#"{{"protocol":"filament-analyze.claim-selection/v1","spans":[{{"authority":"agent-ix","identity":"rules/a.qsl","start":{start},"end":{end}}}]}}"#
            )
            .into_bytes(),
        ));
    }
    malformed.push(("oversized selection", vec![b' '; 1024 * 1024 + 1]));

    for (label, bytes) in malformed {
        let path = selection.fixture.path("selection.json");
        fs::write(&path, bytes).expect("selection bytes");
        let mut arguments = selection.fixture.arguments();
        arguments.extend(["--select".into(), path.into()]);
        let run = selection.fixture.run_with(&arguments);
        assert_eq!(run.code, Some(2), "{label}: {}", run.stderr);
        assert!(
            !run.stderr.contains("empty selection"),
            "{label}: {}",
            run.stderr
        );
        selection.fixture.assert_no_report();
    }

    let mut repeated = selection.fixture.arguments();
    repeated.extend([
        "--select".into(),
        selection.fixture.path("selection.json").into(),
        "--select".into(),
        selection.fixture.path("selection.json").into(),
    ]);
    assert_eq!(selection.fixture.run_with(&repeated).code, Some(2));
    let mut missing_value = selection.fixture.arguments();
    missing_value.push("--select".into());
    assert_eq!(selection.fixture.run_with(&missing_value).code, Some(2));
}

#[trace("FR-009-AC-30")]
#[test]
fn partial_claim_overlap_is_refused_even_with_another_matching_span() {
    let selection = selection_fixture();
    let (start, end) = selection.first_claim;
    let partial = selection_span(start + 1, end);
    let cases = [
        json!([partial.clone()]),
        json!([
            selection_span(start, end),
            selection_span(selection.first_parameter.0, selection.first_parameter.1),
            partial.clone(),
        ]),
        json!([
            selection_span(selection.second_claim.0, selection.second_claim.1),
            selection_span(selection.second_parameter.0, selection.second_parameter.1),
            partial,
        ]),
    ];
    for spans in cases {
        let path = selection_file(&selection.fixture, spans);
        let mut arguments = selection.fixture.arguments();
        arguments.extend(["--select".into(), path.into()]);
        let run = selection.fixture.run_with(&arguments);
        assert_eq!(run.code, Some(2), "{}", run.stderr);
        assert!(
            run.stderr
                .contains(&format!("overlaps {}", selection.first_id)),
            "{}",
            run.stderr
        );
        assert!(run.stderr.contains("overlaps"), "{}", run.stderr);
        selection.fixture.assert_no_report();
    }
}

#[trace("FR-009-AC-32")]
#[test]
fn selecting_every_claim_changes_only_the_selection_member() {
    let selection = selection_fixture();
    selection.fixture.both("unknown\n", "unknown\n");
    let default = selection.fixture.run();
    assert_eq!(default.code, Some(3), "{}", default.stderr);
    let default_report = normalised(&fs::read(selection.fixture.output()).expect("default report"));
    fs::remove_file(selection.fixture.output()).expect("remove default report");

    let spans = json!([selection_span(0, 1000),]);
    let path = selection_file(&selection.fixture, spans);
    let mut arguments = selection.fixture.arguments();
    arguments.extend(["--select".into(), path.into()]);
    let selected = selection.fixture.run_with(&arguments);
    assert_eq!(selected.code, Some(3), "{}", selected.stderr);
    let mut selected_report =
        normalised(&fs::read(selection.fixture.output()).expect("selected report"));
    let selection_member = selected_report
        .as_object_mut()
        .expect("report object")
        .remove("selection")
        .expect("selection member");
    assert_eq!(
        selection_member["selected"],
        json!([selection.first_id, selection.second_id])
    );
    assert_eq!(selection_member["excluded"], json!([]));
    assert_eq!(selected_report, default_report);
}

#[trace("FR-009-AC-33")]
#[test]
fn excluded_non_boolean_untranslatable_and_regionless_claims_are_not_translated() {
    let mut document = Document::new();
    document.source_identity = "rules/a.qsl".to_owned();
    let approved_parameter = document.parameter();
    let approved_claim = document.claim(&approved_parameter);
    let approved_region = document.region(&approved_claim);
    let approved_parameter_region = document.region(&approved_parameter);

    let first = document.op(AND, &[&approved_parameter, &approved_parameter]);
    let second = document.op(AND, &[&first, &approved_parameter]);
    document.set_body(
        &first,
        json!({
            "term": "application",
            "operator": "binary",
            "operation": {"identity": AND, "laws": [], "mode": null, "member": null, "leaves": []},
            "result_type": id("scalar_type/boolean"),
            "arguments": [
                common::reference(&second),
                common::reference(&approved_parameter)
            ],
        }),
        &[&second, &approved_parameter],
    );
    for label in [&first, &second] {
        let node = document
            .nodes
            .iter_mut()
            .find(|node| node["node_id"] == id(label))
            .expect("cycle node");
        node["recursion_group"] = json!("excluded-cycle");
    }
    let cyclic_claim = document.claim(&second);

    let range = document.range();
    let integer_parameter = document.parameter_of(&range);
    let non_boolean_claim = document.claim(&integer_parameter);
    let no_region_claim = document.claim(&approved_parameter);
    let package = document.bytes(&[
        &approved_claim,
        &cyclic_claim,
        &non_boolean_claim,
        &no_region_claim,
    ]);
    let admitted = common::read_package(&package);
    let node = common::index(&admitted, &no_region_claim);
    let mut value: Value = serde_json::from_slice(&package).expect("package JSON");
    value["semantic_graph"]["nodes"][node]["occurrences"] = json!([]);
    value["source_map"]["nodes"][node]["occurrences"] = json!([]);
    value["source_map"]["requested_items"][3]["occurrences"] = json!([]);
    let package = serde_json::to_vec(&value).expect("regionless package");
    let fixture = Fixture::new(&package);
    fixture.both("unknown\n", "unknown\n");
    let path = selection_file(
        &fixture,
        json!([
            selection_span(approved_region.0, approved_region.1),
            selection_span(approved_parameter_region.0, approved_parameter_region.1),
        ]),
    );
    let mut arguments = fixture.arguments();
    arguments.extend(["--select".into(), path.into()]);
    let run = fixture.run_with(&arguments);
    assert_eq!(run.code, Some(3), "{}", run.stderr);
    assert_eq!(
        fixture.report()["selection"]["selected"]
            .as_array()
            .unwrap()
            .len(),
        1
    );

    fs::remove_file(fixture.output()).expect("remove selected report");
    let run = fixture.run();
    assert_eq!(run.code, Some(2), "{}", run.stderr);
    fixture.assert_no_report();
}

#[trace("FR-009-AC-35")]
#[test]
fn reached_expression_nodes_must_be_covered_without_selecting_intermediate_claims() {
    let fixture = Fixture::new(b"pending package");
    let mut document = Document::new();
    document.source_identity = "rules/a.qsl".to_owned();
    let parameter = document.parameter();
    let intermediate = document.claim(&parameter);
    let expression = document.op(AND, &[&parameter, &parameter]);
    let selected_claim = document.claim(&expression);
    let claim_region = document.region(&selected_claim);
    let expression_region = document.region(&expression);
    let parameter_region = document.region(&parameter);
    let package_bytes = document.bytes(&[&intermediate, &selected_claim]);
    let package = common::read_package(&package_bytes);
    let selected_node = common::index(&package, &selected_claim);
    let expression_node = common::index(&package, &expression);
    let intermediate_node = common::index(&package, &intermediate);
    fs::write(&fixture.package, package_bytes).expect("write package");
    fixture.both("unknown\n", "unknown\n");

    let incomplete = selection_file(
        &fixture,
        json!([
            selection_span(claim_region.0, claim_region.1),
            selection_span(parameter_region.0, parameter_region.1),
        ]),
    );
    let mut arguments = fixture.arguments();
    arguments.extend(["--select".into(), incomplete.into()]);
    let run = fixture.run_with(&arguments);
    assert_eq!(run.code, Some(2), "{}", run.stderr);
    assert!(
        run.stderr.contains(&format!("claim-{selected_node}")),
        "{}",
        run.stderr
    );
    assert!(
        run.stderr
            .contains(&format!("semantic_graph.nodes[{expression_node}]")),
        "{}",
        run.stderr
    );
    fixture.assert_no_report();

    let complete = selection_file(
        &fixture,
        json!([
            selection_span(claim_region.0, claim_region.1),
            selection_span(expression_region.0, expression_region.1),
            selection_span(parameter_region.0, parameter_region.1),
        ]),
    );
    let mut arguments = fixture.arguments();
    arguments.extend(["--select".into(), complete.into()]);
    let run = fixture.run_with(&arguments);
    assert_eq!(run.code, Some(3), "{}", run.stderr);
    let report = fixture.report();
    assert_eq!(
        report["selection"]["selected"],
        json!([format!("claim-{selected_node}")])
    );
    assert_eq!(
        report["selection"]["excluded"],
        json!([format!("claim-{intermediate_node}")])
    );

    let no_region_fixture = Fixture::new(b"pending package");
    let mut document = Document::new();
    document.source_identity = "rules/a.qsl".to_owned();
    let parameter = document.parameter();
    let selected_claim = document.claim(&parameter);
    let claim_region = document.region(&selected_claim);
    let bytes = document.bytes(&[&selected_claim]);
    let admitted = common::read_package(&bytes);
    let parameter_node = common::index(&admitted, &parameter);
    let mut value: Value = serde_json::from_slice(&bytes).expect("package JSON");
    value["semantic_graph"]["nodes"][parameter_node]["occurrences"] = json!([]);
    value["source_map"]["nodes"][parameter_node]["occurrences"] = json!([]);
    fs::write(
        &no_region_fixture.package,
        serde_json::to_vec(&value).expect("package without parameter region"),
    )
    .expect("write package");
    let path = selection_file(
        &no_region_fixture,
        json!([selection_span(claim_region.0, claim_region.1)]),
    );
    let mut arguments = no_region_fixture.arguments();
    arguments.extend(["--select".into(), path.into()]);
    let run = no_region_fixture.run_with(&arguments);
    assert_eq!(run.code, Some(2), "{}", run.stderr);
    assert!(run.stderr.contains("claim-0"), "{}", run.stderr);
    assert!(
        run.stderr
            .contains(&format!("semantic_graph.nodes[{parameter_node}]")),
        "{}",
        run.stderr
    );
    no_region_fixture.assert_no_report();
}

#[trace("FR-009-AC-36")]
#[test]
fn empty_selection_has_its_own_usage_cause() {
    let selection = selection_fixture();
    let path = selection_file(&selection.fixture, json!([]));
    let mut arguments = selection.fixture.arguments();
    arguments.extend(["--select".into(), path.into()]);
    let empty = selection.fixture.run_with(&arguments);
    assert_eq!(empty.code, Some(2));
    assert!(empty.stderr.contains("empty selection"), "{}", empty.stderr);

    let malformed = selection_file(
        &selection.fixture,
        json!([{"authority":"", "identity":"rules/a.qsl", "start":1, "end":2}]),
    );
    let mut arguments = selection.fixture.arguments();
    arguments.extend(["--select".into(), malformed.into()]);
    let malformed = selection.fixture.run_with(&arguments);
    assert_eq!(malformed.code, Some(2));
    assert!(
        !malformed.stderr.contains("empty selection"),
        "{}",
        malformed.stderr
    );

    let directory = selection.fixture.path("selection-directory");
    fs::create_dir(&directory).expect("selection directory");
    let mut arguments = selection.fixture.arguments();
    arguments.extend(["--select".into(), directory.into()]);
    let directory = selection.fixture.run_with(&arguments);
    assert_eq!(directory.code, Some(2), "{}", directory.stderr);
    assert!(
        directory.stderr.contains("not a regular file"),
        "{}",
        directory.stderr
    );

    let missing = selection.fixture.path("missing-selection.json");
    let mut arguments = selection.fixture.arguments();
    arguments.extend(["--select".into(), missing.into()]);
    let missing = selection.fixture.run_with(&arguments);
    assert_eq!(missing.code, Some(2), "{}", missing.stderr);
    assert!(
        missing.stderr.contains("No such file"),
        "{}",
        missing.stderr
    );
}

#[trace("FR-009-AC-29")]
#[test]
fn selection_filters_consistency_and_is_recorded_in_the_report() {
    let selection = selection_fixture();
    selection.fixture.both("unknown\n", "unknown\n");
    let spans = json!([
        selection_span(selection.first_claim.0, selection.first_claim.1),
        selection_span(selection.first_parameter.0, selection.first_parameter.1),
    ]);
    let selection_path = selection_file(&selection.fixture, spans);
    let mut arguments = selection.fixture.arguments();
    arguments.extend(["--select".into(), selection_path.into()]);

    let run = selection.fixture.run_with(&arguments);
    assert_eq!(run.code, Some(3), "{}", run.stderr);
    let report = selection.fixture.report();
    assert_eq!(report["selection"]["selected"], json!([selection.first_id]));
    assert_eq!(
        report["selection"]["excluded"],
        json!([selection.second_id])
    );
    assert_eq!(
        report["assertions"].as_array().expect("assertions").len(),
        1
    );
}

#[test]
fn contradiction_maps_approved_claims_left_and_selected_draft_right() {
    let selection = selection_fixture();
    selection.fixture.both("unknown\n", "unknown\n");
    let spans = json!([
        selection_span(selection.first_claim.0, selection.first_claim.1),
        selection_span(selection.first_parameter.0, selection.first_parameter.1),
    ]);
    let selection_path = selection_file(&selection.fixture, spans);
    let mut arguments = selection.fixture.arguments();
    arguments.extend([
        "--select".into(),
        selection_path.into(),
        "--request".into(),
        "contradiction".into(),
    ]);

    let run = selection.fixture.run_with(&arguments);
    assert_eq!(run.code, Some(3), "{}", run.stderr);
    let report = selection.fixture.report();
    assert_eq!(report["analysisKind"], "contradiction");
    let assertions = report["assertions"].as_array().expect("assertions");
    assert_eq!(assertions.len(), 2);
    let left = assertions
        .iter()
        .find(|assertion| assertion["role"] == "left")
        .expect("approved left claim");
    let right = assertions
        .iter()
        .find(|assertion| assertion["role"] == "right")
        .expect("draft right claim");
    assert!(left["clause"]
        .as_str()
        .unwrap()
        .ends_with(&selection.second_id));
    assert!(right["clause"]
        .as_str()
        .unwrap()
        .ends_with(&selection.first_id));
}

#[test]
fn redundancy_maps_the_single_draft_candidate_and_approved_peer() {
    let selection = selection_fixture();
    selection.fixture.both("unknown\n", "unknown\n");
    let spans = json!([
        selection_span(selection.first_claim.0, selection.first_claim.1),
        selection_span(selection.first_parameter.0, selection.first_parameter.1),
    ]);
    let selection_path = selection_file(&selection.fixture, spans);
    let mut arguments = selection.fixture.arguments();
    arguments.extend([
        "--select".into(),
        selection_path.into(),
        "--request".into(),
        "redundancy".into(),
    ]);

    let run = selection.fixture.run_with(&arguments);
    assert_eq!(run.code, Some(3), "{}", run.stderr);
    let report = selection.fixture.report();
    assert_eq!(report["analysisKind"], "redundancy");
    let assertions = report["assertions"].as_array().expect("assertions");
    assert_eq!(assertions.len(), 2);
    let peer = assertions
        .iter()
        .find(|assertion| assertion["role"] == "peer")
        .expect("approved peer claim");
    let candidate = assertions
        .iter()
        .find(|assertion| assertion["role"] == "candidate")
        .expect("draft candidate claim");
    assert!(peer["clause"]
        .as_str()
        .unwrap()
        .ends_with(&selection.second_id));
    assert!(candidate["clause"]
        .as_str()
        .unwrap()
        .ends_with(&selection.first_id));
}

#[test]
fn non_default_requests_require_a_usable_selection() {
    let built = consistent();
    let fixture = Fixture::new(&built.bytes);
    let mut no_selection = fixture.arguments();
    no_selection.extend(["--request".into(), "contradiction".into()]);
    let run = fixture.run_with(&no_selection);
    assert_eq!(run.code, Some(2), "{}", run.stderr);
    assert!(run.stderr.contains("require --select"), "{}", run.stderr);

    let selection = selection_fixture();
    let both = json!([
        selection_span(selection.first_claim.0, selection.first_claim.1),
        selection_span(selection.first_parameter.0, selection.first_parameter.1),
        selection_span(selection.second_claim.0, selection.second_claim.1),
        selection_span(selection.second_parameter.0, selection.second_parameter.1),
    ]);
    let selection_path = selection_file(&selection.fixture, both);
    let mut ambiguous = selection.fixture.arguments();
    ambiguous.extend([
        "--select".into(),
        selection_path.into(),
        "--request".into(),
        "redundancy".into(),
    ]);
    let run = selection.fixture.run_with(&ambiguous);
    assert_eq!(run.code, Some(2), "{}", run.stderr);
    assert!(run.stderr.contains("one draft claim"), "{}", run.stderr);
    selection.fixture.assert_no_report();
}

#[trace("FR-009-AC-1", "FR-009-AC-16", "FR-009-AC-2")]
#[test]
fn two_requests_per_engine_version_then_query_and_a_satisfied_report() {
    let built = consistent();
    let fixture = Fixture::new(&built.bytes);
    fixture.both(&built.model, &built.model);
    let run = fixture.run();
    assert_eq!(run.code, Some(0), "{}", run.stderr);

    let requests = fixture.requests();
    assert_eq!(requests.len(), 4);
    let sequence = requests
        .iter()
        .map(|request| {
            (
                request["producer"]["name"].as_str().expect("engine"),
                request["stdin"]["kind"].as_str().expect("stdin"),
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(
        sequence,
        [
            ("z3", "null"),
            ("z3", "input_artifact"),
            ("cvc5", "null"),
            ("cvc5", "input_artifact")
        ]
    );
    for (index, request) in requests.iter().enumerate() {
        let (path, version, arguments, text) = if index < 2 {
            (fixture.z3(), VERSION_Z3, ["-version"].as_slice(), "-in")
        } else {
            (
                fixture.cvc5(),
                VERSION_CVC5,
                ["--version"].as_slice(),
                "--lang=smt2",
            )
        };
        let is_query = index % 2 == 1;
        assert_eq!(
            request["protocol"],
            "engineering-assurance.producer-execution-request/v1"
        );
        assert_eq!(request["procedure"], "direct");
        assert_eq!(
            request["producer"]["executable"],
            path.to_str().expect("utf8")
        );
        assert_eq!(
            request["caller"],
            json!({"kind": "filament-analyze.run-package", "version": "1"})
        );
        assert_eq!(request["environment"], json!({}));
        assert_eq!(request["outputs"], json!([]));
        assert_eq!(request["containment"]["profile"], "process-group-v1");
        assert_eq!(request["cancellation"], json!({"kind": "disabled"}));
        assert_eq!(
            request["response"],
            json!({
                "protocol": {"kind": "engineering-assurance.process-evidence", "version": "1"},
                "adapter": {"kind": "engineering-assurance.cli-process-evidence-adapter", "version": "1"},
                "exitCodes": {"kind": "any"},
            })
        );
        if is_query {
            assert_eq!(request["producer"]["version"], version.trim());
            assert_eq!(request["arguments"][0]["value"], text);
            assert_eq!(request["inputs"][0]["role"], "query");
            assert_eq!(request["stdin"]["role"], "query");
        } else {
            assert_eq!(request["producer"]["version"], "unprobed");
            assert_eq!(request["arguments"][0]["value"], arguments[0]);
            assert_eq!(request["inputs"], json!([]));
        }
        let budget = &request["budget"];
        assert!(budget["maxStdoutBytes"].as_u64().expect("stdout") <= 8_388_608);
        assert!(budget["maxStderrBytes"].as_u64().expect("stderr") <= 8_388_608);
        assert_eq!(budget["maxOutputArtifacts"], 0);
        assert_eq!(budget["maxOutputBytes"], 0);
        assert_eq!(budget["maxDescendants"], 16);
        assert_eq!(budget["maxConcurrency"], 1);
    }

    let bytes = fs::read(fixture.output()).expect("report");
    assert_eq!(
        validate_report_document(&bytes),
        Ok(DifferentialDisposition::Agreement)
    );
    assert_eq!(
        fixture.report()["differential"]["agreedStatus"],
        "satisfied"
    );
    fixture.assert_no_leftovers();
}

#[trace("FR-009-AC-3")]
#[test]
fn both_unsat_for_contradictory_clauses_is_refuted() {
    let built = contradictory();
    let fixture = Fixture::new(&built.bytes);
    fixture.both("unsat\n", "unsat\n");
    let run = fixture.run();
    assert_eq!(run.code, Some(1), "{}", run.stderr);
    assert_eq!(fixture.report()["differential"]["agreedStatus"], "refuted");
}

/// One row of the result-state table, applied to the Z3 query invocation.
struct Row {
    result: String,
    outcome: &'static str,
    disposition: &'static str,
}

fn rows() -> Vec<Row> {
    let row = |result: String, outcome, disposition| Row {
        result,
        outcome,
        disposition,
    };
    let failed = |reason: Value| state_only(json!({"kind": "failed", "reason": reason}));
    vec![
        row(
            state_only(json!({"kind": "unavailable"})),
            "missing-executable",
            "unavailable",
        ),
        row(
            state_only(json!({"kind": "timed_out"})),
            "timed-out",
            "inconclusive",
        ),
        row(
            failed(json!({"kind": "exit_code", "value": 3})),
            "nonzero-exit",
            "inconclusive",
        ),
        row(
            failed(json!({"kind": "signal", "value": 9})),
            "signaled",
            "inconclusive",
        ),
        row(
            failed(json!({"kind": "stdout_too_large"})),
            "stdout-limit",
            "inconclusive",
        ),
        row(
            failed(json!({"kind": "stderr_too_large"})),
            "stderr-limit",
            "inconclusive",
        ),
        row(
            failed(json!({"kind": "observation"})),
            "io-error",
            "inconclusive",
        ),
        row(
            failed(json!({"kind": "output_artifact"})),
            "io-error",
            "inconclusive",
        ),
        row(
            state_only(json!({"kind": "refused", "reason": "executable"})),
            "spawn-error",
            "inconclusive",
        ),
        row(
            state_only(json!({"kind": "malformed_response"})),
            "malformed-output",
            "inconclusive",
        ),
        row(
            state_only(json!({"kind": "containment_failure"})),
            "cleanup-failed",
            "inconclusive",
        ),
        row(
            state_only(json!({"kind": "cancelled"})),
            "cancelled",
            "inconclusive",
        ),
    ]
}

#[trace("FR-009-AC-4", "FR-009-AC-17")]
#[test]
fn every_non_completed_state_records_its_table_outcome_and_exits_three() {
    let built = consistent();
    for row in rows() {
        let fixture = Fixture::new(&built.bytes);
        fixture.both(&built.model, &built.model);
        fixture.answer("z3", "query", &row.result);
        let run = fixture.run();
        assert_eq!(run.code, Some(3), "{}: {}", row.outcome, run.stderr);
        let report = fixture.report();
        let bytes = fs::read(fixture.output()).expect("report");
        validate_report_document(&bytes).expect("valid report");
        assert_eq!(outcomes(&report).0, row.outcome);
        assert_eq!(report["differential"]["disposition"], row.disposition);
        assert_eq!(report["engines"].as_array().expect("engines").len(), 2);
        assert_eq!(outcomes(&report).1, "sat");
    }
}

#[trace("FR-009-AC-4", "FR-009-AC-17")]
#[test]
fn completed_results_are_classified_and_opposite_answers_exit_three() {
    let built = consistent();
    let cases = [
        ("unknown\n", "unknown"),
        ("(error \"boom\")\n", "solver-error"),
        ("garbage\n", "malformed-output"),
    ];
    for (stdout, outcome) in cases {
        let fixture = Fixture::new(&built.bytes);
        fixture.both(stdout, &built.model);
        let run = fixture.run();
        assert_eq!(run.code, Some(3), "{}", run.stderr);
        assert_eq!(outcomes(&fixture.report()).0, outcome);
    }
    let fixture = Fixture::new(&built.bytes);
    fixture.both(&built.model, "unsat\n");
    assert_eq!(fixture.run().code, Some(3));
    assert_eq!(
        fixture.report()["differential"]["disposition"],
        "disagreement"
    );
    assert_eq!(outcomes(&fixture.report()), ("sat".into(), "unsat".into()));
}

#[trace("FR-009-AC-16")]
#[test]
fn a_bad_or_empty_version_skips_the_query_and_exits_three() {
    let built = consistent();
    let bad_versions = [
        (state_only(json!({"kind": "timed_out"})), "timed-out"),
        (
            state_only(json!({"kind": "unavailable"})),
            "missing-executable",
        ),
        (completed(0, "  \n", ""), "identity-error"),
    ];
    for (version, outcome) in bad_versions {
        let fixture = Fixture::new(&built.bytes);
        fixture.both(&built.model, &built.model);
        fixture.answer("cvc5", "version", &version);
        let run = fixture.run();
        assert_eq!(run.code, Some(3), "{}", run.stderr);
        // z3 version, z3 query, cvc5 version only.
        assert_eq!(fixture.requests().len(), 3);
        assert_eq!(outcomes(&fixture.report()).1, outcome);
        assert!(!fixture.path("stub/data/cvc5.staged").exists());
    }
}

#[trace("FR-009-AC-5")]
#[test]
fn an_unusable_producer_execute_exits_four_without_a_report() {
    let built = consistent();
    let line_ok = completed(0, "unknown\n", "");
    let scenarios: Vec<Scenario> = vec![
        (
            "cannot start",
            Box::new(|fixture| fs::remove_file(fixture.producer()).expect("remove")),
        ),
        (
            "exit status 1",
            Box::new(|fixture| fixture.exit_after("z3", "version", 1)),
        ),
        (
            "exit status 2",
            Box::new(|fixture| fixture.exit_after("z3", "version", 2)),
        ),
        (
            "exit status 7 on the query",
            Box::new(|fixture| fixture.exit_after("cvc5", "query", 7)),
        ),
        (
            "signal",
            Box::new(|fixture| fixture.signal("z3", "version")),
        ),
        (
            "no line",
            Box::new(|fixture| fixture.answer("z3", "version", "")),
        ),
        (
            "two lines",
            Box::new({
                let two = format!("{line_ok}\n{line_ok}");
                move |fixture| fixture.answer("z3", "version", &two)
            }),
        ),
        (
            "garbage",
            Box::new(|fixture| fixture.answer("z3", "version", "not json")),
        ),
        (
            "another document",
            Box::new(|fixture| fixture.answer("z3", "version", "{\"protocol\":\"other\"}")),
        ),
    ];
    for (label, apply) in scenarios {
        let fixture = Fixture::new(&built.bytes);
        apply(&fixture);
        if label == "cannot start" {
            // The directory is still needed to hold the output; the stub is gone.
            assert!(!fixture.producer().exists());
        }
        let run = fixture.run();
        assert_eq!(run.code, Some(4), "{label}: {}", run.stderr);
        assert!(
            run.stderr.contains("producer-execute"),
            "{label}: {}",
            run.stderr
        );
        fixture.assert_no_report();
    }
}

/// A named change to a fixture.
type Scenario = (&'static str, Box<dyn Fn(&Fixture)>);

fn overlong() -> OsString {
    OsString::from(format!("/{}", "a".repeat(5000)))
}

fn non_utf8() -> OsString {
    OsString::from_vec(vec![b'/', b'a', 0xff])
}

#[trace("FR-009-AC-6")]
#[test]
fn refused_packages_exit_two_without_a_report() {
    let built = consistent();

    let mut cases: Vec<(&str, Vec<u8>)> = Vec::new();
    cases.push(("garbage", b"not a package".to_vec()));
    cases.push(("truncated", built.bytes[..built.bytes.len() / 2].to_vec()));

    let mut value: Value = serde_json::from_slice(&built.bytes).expect("json");
    value["package"]["source"]["lock"]["dependency_selections"] =
        json!([{"identity": "library/dependency"}]);
    cases.push(("dependency", serde_json::to_vec(&value).expect("bytes")));

    let mut document = Document::new();
    let p = document.parameter();
    cases.push(("no analysis claim", document.bytes(&[&p])));

    let mut document = Document::new();
    let range = document.range();
    let number = document.parameter_of(&range);
    let claim = document.claim(&number);
    cases.push(("non-Boolean claim", document.bytes(&[&claim])));

    let limit = 1_048_576;
    let mut padded = built.bytes.clone();
    padded.resize(limit + 1, b' ');
    cases.push(("one byte over the limit", padded));

    for (label, bytes) in cases {
        let fixture = Fixture::new(&bytes);
        let run = fixture.run();
        assert_eq!(run.code, Some(2), "{label}: {}", run.stderr);
        assert!(run.stderr.contains("--package"), "{label}: {}", run.stderr);
        if label == "dependency" {
            assert!(
                run.stderr.contains("dependency_selections"),
                "{label}: {}",
                run.stderr
            );
        }
        assert!(fixture.requests().is_empty(), "{label}");
        fixture.assert_no_report();
    }

    let fixture = Fixture::new(&built.bytes);
    fs::remove_file(&fixture.package).expect("remove");
    let run = fixture.run();
    assert_eq!(run.code, Some(2), "{}", run.stderr);
    fixture.assert_no_report();
}

#[trace("FR-009-AC-6")]
#[test]
fn a_document_at_the_limit_is_not_refused_for_size() {
    let built = consistent();
    let mut padded = built.bytes.clone();
    padded.resize(1_048_576, b' ');
    let fixture = Fixture::new(&padded);
    let run = fixture.run();
    assert_eq!(run.code, Some(2), "{}", run.stderr);
    assert!(!run.stderr.contains("read limit"), "{}", run.stderr);
    // The reader got past the size check and refused the padding as non-canonical.
    assert!(run.stderr.contains("NoncanonicalWire"), "{}", run.stderr);
    padded.push(b' ');
    let fixture = Fixture::new(&padded);
    let run = fixture.run();
    assert_eq!(run.code, Some(2), "{}", run.stderr);
    assert!(run.stderr.contains("read limit"), "{}", run.stderr);
}

#[trace("FR-009-AC-7")]
#[test]
fn bad_arguments_exit_two_with_the_usage() {
    let built = consistent();
    let fixture = Fixture::new(&built.bytes);
    let mut cases: Vec<(&str, Vec<OsString>)> = Vec::new();
    for option in [
        "--package",
        "--output",
        "--z3",
        "--cvc5",
        "--producer-execute",
    ] {
        cases.push((option, fixture.without(option)));
        let mut repeated = fixture.arguments();
        repeated.push(option.into());
        repeated.push("/x".into());
        cases.push((option, repeated));
        let mut valueless = fixture.arguments();
        let position = valueless
            .iter()
            .position(|argument| argument == option)
            .expect("option");
        valueless.truncate(position + 1);
        cases.push((option, valueless));
    }
    let mut unknown = fixture.arguments();
    unknown.push("--unknown".into());
    unknown.push("x".into());
    cases.push(("unknown", unknown));
    let mut unknown_request = fixture.arguments();
    unknown_request.extend(["--request".into(), "unknown".into()]);
    cases.push(("unknown request", unknown_request));
    let mut valueless_request = fixture.arguments();
    valueless_request.push("--request".into());
    cases.push(("valueless request", valueless_request));
    cases.push(("none", vec!["run-package".into()]));
    for (label, arguments) in cases {
        let run = fixture.run_with(&arguments);
        assert_eq!(run.code, Some(2), "{label}: {}", run.stderr);
        assert!(run.stderr.contains("usage:"), "{label}: {}", run.stderr);
        assert!(run.stdout.is_empty());
        fixture.assert_no_report();
    }
}

#[trace("FR-009-AC-8")]
#[test]
fn bad_executable_paths_exit_two() {
    let built = consistent();
    let fixture = Fixture::new(&built.bytes);
    let bad = [
        ("relative", OsString::from("relative/solver")),
        ("empty", OsString::new()),
        ("non-utf8", non_utf8()),
        ("over-long", overlong()),
    ];
    for option in ["--z3", "--cvc5", "--producer-execute"] {
        for (label, value) in &bad {
            let run = fixture.run_with(&fixture.with(option, value.clone()));
            assert_eq!(run.code, Some(2), "{option} {label}: {}", run.stderr);
            assert!(fixture.requests().is_empty());
            fixture.assert_no_report();
        }
    }
}

#[trace("FR-009-AC-9")]
#[test]
fn a_bad_destination_exits_four_and_keeps_existing_bytes() {
    let built = consistent();
    let fixture = Fixture::new(&built.bytes);
    fixture.both(&built.model, &built.model);

    fs::write(fixture.output(), b"keep").expect("existing");
    let run = fixture.run();
    assert_eq!(run.code, Some(4), "{}", run.stderr);
    assert_eq!(fs::read(fixture.output()).expect("bytes"), b"keep");
    assert_eq!(directory_names(&fixture.path("out")), ["report.json"]);
    fs::remove_file(fixture.output()).expect("remove");

    let missing = fixture.path("missing/report.json");
    let run = fixture.run_with(&fixture.with("--output", missing.clone()));
    assert_eq!(run.code, Some(4), "{}", run.stderr);
    assert!(!missing.exists());

    let out = fixture.path("out");
    fs::set_permissions(&out, fs::Permissions::from_mode(0o500)).expect("read-only");
    if fs::write(out.join("probe"), b"").is_ok() {
        eprintln!("skipped the unwritable-parent case: directory permissions are not enforced");
    } else {
        let run = fixture.run();
        assert_eq!(run.code, Some(4), "{}", run.stderr);
        assert!(directory_names(&out).is_empty());
    }
    fs::set_permissions(&out, fs::Permissions::from_mode(0o700)).expect("restore");
}

#[trace("FR-009-AC-11", "FR-009-AC-12")]
#[test]
fn identical_inputs_give_identical_bytes_and_leave_no_trace() {
    let built = consistent();
    let fixture = Fixture::new(&built.bytes);
    fixture.both(&built.model, &built.model);
    let before = (
        directory_names(&fixture.path("pkg")),
        fs::read(&fixture.package).expect("package"),
        fs::metadata(&fixture.package)
            .expect("metadata")
            .modified()
            .expect("mtime"),
    );

    assert_eq!(fixture.run().code, Some(0));
    let first = fs::read(fixture.output()).expect("report");
    fs::remove_file(fixture.output()).expect("remove");
    // The second run measures a different execution time: only `elapsedMs` may differ.
    fixture.retime(3_000_000, 7_000_000);
    assert_eq!(fixture.run().code, Some(0));
    let second = fs::read(fixture.output()).expect("report");
    assert_ne!(first, second, "elapsedMs carries the measured time");
    assert_eq!(normalised(&first), normalised(&second));

    let after = (
        directory_names(&fixture.path("pkg")),
        fs::read(&fixture.package).expect("package"),
        fs::metadata(&fixture.package)
            .expect("metadata")
            .modified()
            .expect("mtime"),
    );
    assert_eq!(before, after);
    fixture.assert_no_leftovers();

    // Every invocation saw a capability root of mode 0700 holding the staged query.
    let modes = fs::read_to_string(fixture.path("stub/data/modes.log")).expect("modes");
    assert_eq!(modes.lines().count(), 8);
    assert!(modes.lines().all(|mode| mode == "700"), "{modes}");
    let staged = fs::read_to_string(fixture.path("stub/data/z3.staged")).expect("staged query");
    assert!(staged.contains("(check-sat)"), "{staged}");
}

#[trace("TC-030", "FR-034-AC-4")]
#[test]
fn quantified_package_runs_publish_deterministic_report_bytes() {
    let mut document = Document::new();
    let binder = document.parameter();
    let first = document.literal(true);
    let collection = document.collection_literal("sequence", "scalar_type/boolean", &[&first]);
    let quantified =
        document.quantifier("filament.op.collection.forall", &collection, "p1", &binder);
    let claim = document.claim(&quantified);
    let bytes = document.bytes(&[&claim]);
    let fixture = Fixture::new(&bytes);
    fixture.both("sat\n(model)\n", "sat\n(model)\n");

    assert_eq!(fixture.run().code, Some(0));
    let first_report = fs::read(fixture.output()).expect("first report");
    fs::remove_file(fixture.output()).expect("remove first report");
    fixture.retime(3_000_000, 7_000_000);
    assert_eq!(fixture.run().code, Some(0));
    let second_report = fs::read(fixture.output()).expect("second report");
    assert_ne!(
        first_report, second_report,
        "elapsedMs carries measured time"
    );
    assert_eq!(normalised(&first_report), normalised(&second_report));
}

#[trace("TC-030", "FR-034-AC-2")]
#[test]
fn a_text_quantifier_domain_exits_two_without_a_solver_call_or_report() {
    let mut document = Document::new();
    document.source_identity = "spec/fixtures/text-domain.qsl".to_owned();
    let text = document.unsupported_type("text");
    let _binder = document.parameter_of(&text);
    let collection = document.collection_literal("sequence", &text, &[]);
    let predicate = document.literal(true);
    let quantifier = document.quantifier(
        "filament.op.collection.forall",
        &collection,
        "p1",
        &predicate,
    );
    let source_region = document.region(&quantifier);
    let claim = document.claim(&quantifier);
    let bytes = document.bytes(&[&claim]);
    let fixture = Fixture::new(&bytes);

    let run = fixture.run();
    assert_eq!(run.code, Some(2), "{}", run.stderr);
    let node = format!("semantic_graph.nodes[{}]", node_index(&bytes, &quantifier));
    let expected = format!(
        "--package: {node}: text-domain: the collection element type is not supported \
         (source agent-ix:{} bytes {}..{})",
        document.source_identity, source_region.0, source_region.1
    );
    assert!(run.stderr.contains(&expected), "{expected}: {}", run.stderr);
    assert!(fixture.requests().is_empty(), "a solver was invoked");
    fixture.assert_no_report();
}

#[trace("FR-009-AC-12")]
#[test]
fn the_capability_root_is_gone_after_every_exit_path() {
    let built = consistent();
    let satisfied = Fixture::new(&built.bytes);
    satisfied.both(&built.model, &built.model);
    assert_eq!(satisfied.run().code, Some(0));
    satisfied.assert_no_leftovers();

    let non_conclusive = Fixture::new(&built.bytes);
    assert_eq!(non_conclusive.run().code, Some(3));
    non_conclusive.assert_no_leftovers();

    let usage = Fixture::new(&built.bytes);
    assert_eq!(usage.run_with(&usage.without("--z3")).code, Some(2));
    usage.assert_no_report();

    let failed = Fixture::new(&built.bytes);
    failed.exit_after("cvc5", "version", 9);
    assert_eq!(failed.run().code, Some(4));
    failed.assert_no_report();
    assert_eq!(
        fs::read_to_string(failed.path("stub/data/modes.log"))
            .expect("modes")
            .lines()
            .count(),
        3
    );
}

#[trace("FR-009-AC-13")]
#[test]
fn help_lists_the_arguments_and_exit_classes() {
    let run = finish({
        let mut command = Command::new(BINARY);
        command.args(["run-package", "--help"]).stdin(Stdio::null());
        command
    });
    assert_eq!(run.code, Some(0));
    for text in [
        "--package",
        "--output",
        "--z3",
        "--cvc5",
        "--producer-execute",
        "  0  ",
        "  1  ",
        "  2  ",
        "  3  ",
        "  4  ",
    ] {
        assert!(run.stdout.contains(text), "{text}: {}", run.stdout);
    }
}

#[trace("FR-009-AC-18")]
#[test]
fn a_symlinked_component_exits_two() {
    let built = consistent();
    let fixture = Fixture::new(&built.bytes);
    fixture.both(&built.model, &built.model);
    symlink(fixture.path("solvers"), fixture.path("solver-link")).expect("link");
    symlink(fixture.path("stub"), fixture.path("stub-link")).expect("link");
    symlink(fixture.path("out"), fixture.path("out-link")).expect("link");
    symlink(fixture.producer(), fixture.path("producer-link")).expect("link");

    let cases = [
        ("--z3", fixture.path("solver-link/z3")),
        ("--cvc5", fixture.path("solver-link/cvc5")),
        ("--producer-execute", fixture.path("stub-link/producer")),
        ("--producer-execute", fixture.path("producer-link")),
        ("--output", fixture.path("out-link/report.json")),
    ];
    for (option, value) in cases {
        let run = fixture.run_with(&fixture.with(option, value.clone()));
        assert_eq!(run.code, Some(2), "{option}: {}", run.stderr);
        assert!(
            run.stderr.contains("symbolic link"),
            "{option}: {}",
            run.stderr
        );
        assert!(fixture.requests().is_empty());
        fixture.assert_no_report();
    }
}

/// A package whose claim is a Boolean term the translation cannot lower, and the claim node.
fn untranslatable() -> (Vec<u8>, String) {
    let mut document = Document::new();
    let range = document.range();
    let a = document.parameter_of(&range);
    let b = document.parameter_of(&range);
    let quotient = document.application("filament.op.integer.add", &[&a, &b], &range);
    let less = document.application(
        "filament.op.integer.lt",
        &[&quotient, &b],
        "scalar_type/boolean",
    );
    let claim = document.claim(&less);
    let bytes = with_identity(
        &document.bytes(&[&claim]),
        &quotient,
        "filament.op.integer.div",
    );
    (bytes, quotient)
}

fn node_index(bytes: &[u8], label: &str) -> usize {
    let value: Value = serde_json::from_slice(bytes).expect("json");
    let key = json!({"package": "test/pkg", "path": [label], "ordinal": 0});
    value["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .position(|node| node["node_id"] == key)
        .expect("node")
}

#[trace("FR-009-AC-19", "FR-009-AC-23")]
#[test]
fn an_untranslatable_claim_exits_two_naming_the_node() {
    let (bytes, less) = untranslatable();
    let fixture = Fixture::new(&bytes);
    let run = fixture.run();
    assert_eq!(run.code, Some(2), "{}", run.stderr);
    let node = format!("semantic_graph.nodes[{}]", node_index(&bytes, &less));
    assert!(run.stderr.contains(&node), "{node}: {}", run.stderr);
    assert!(fixture.requests().is_empty());
    fixture.assert_no_report();
}

#[trace("FR-009-AC-23")]
#[test]
fn a_shared_subterm_chain_exits_two_without_hanging() {
    let mut document = Document::new();
    let p = document.parameter();
    let mut top = document.op(AND, &[&p, &p]);
    for _ in 1..61 {
        top = document.op(AND, &[&top, &top]);
    }
    let claim = document.claim(&top);
    let bytes = document.bytes(&[&claim]);
    let fixture = Fixture::new(&bytes);
    let started = std::time::Instant::now();
    let run = fixture.run();
    assert_eq!(run.code, Some(2), "{}", run.stderr);
    assert!(
        run.stderr.contains("semantic_graph.nodes["),
        "{}",
        run.stderr
    );
    assert!(started.elapsed() < std::time::Duration::from_secs(30));
    fixture.assert_no_report();
}

#[trace("FR-009-AC-24")]
#[test]
fn a_claim_without_occurrences_exits_two_naming_the_node() {
    let mut document = Document::new();
    let p = document.parameter();
    let claim = document.claim(&p);
    let bytes = document.bytes(&[&claim]);
    let node = node_index(&bytes, &claim);
    let mut value: Value = serde_json::from_slice(&bytes).expect("json");
    value["semantic_graph"]["nodes"][node]["occurrences"] = json!([]);
    value["source_map"]["nodes"][node]["occurrences"] = json!([]);
    value["source_map"]["requested_items"][0]["occurrences"] = json!([]);
    let fixture = Fixture::new(&serde_json::to_vec(&value).expect("bytes"));
    let run = fixture.run();
    assert_eq!(run.code, Some(2), "{}", run.stderr);
    assert!(
        run.stderr
            .contains(&format!("semantic_graph.nodes[{node}]")),
        "{}",
        run.stderr
    );
    fixture.assert_no_report();
}

#[trace("NFR-003-AC-2")]
#[test]
fn null_stdin_and_an_empty_environment_give_the_same_report_as_open_stdin() {
    let built = consistent();
    let fixture = Fixture::new(&built.bytes);
    fixture.both(&built.model, &built.model);

    // Open stdin: a pipe holding unread data that stays open until the run has ended, and the
    // full environment. A runner that read stdin would block or take the data into its report.
    let mut command = fixture.command(&fixture.arguments());
    let mut child = command
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("spawn");
    let mut open_stdin = child.stdin.take().expect("stdin");
    open_stdin
        .write_all(b"unread data that must not matter\n")
        .expect("write stdin");
    let open_code = child.wait().expect("wait").code();
    drop(open_stdin);
    let open_report = fs::read(fixture.output()).expect("report");
    fs::remove_file(fixture.output()).expect("remove");

    // Null stdin and an empty environment. Network access is not restricted or asserted.
    fixture.retime(3_000_000, 9_000_000);
    let mut command = fixture.command(&fixture.arguments());
    command.env_clear();
    let hermetic = finish(command);
    assert_eq!(open_code, Some(0));
    assert_eq!(open_code, hermetic.code);
    assert_eq!(
        normalised(&open_report),
        normalised(&fs::read(fixture.output()).expect("report"))
    );
}

/// TC-015 case 20: every refusal of case 19 exits `2` naming `semantic_graph.nodes[N]`, with no
/// request and no report, when the binary is spawned over the built package.
#[trace("FR-009-AC-23")]
#[test]
fn every_translation_refusal_exits_two_naming_a_node() {
    let mut cases: Vec<(&str, Vec<u8>)> = Vec::new();

    let mut document = Document::new();
    let p = document.parameter();
    let q = document.parameter();
    let and = document.op(AND, &[&p, &q]);
    let claim = document.claim(&and);
    let bytes = document.bytes(&[&claim]);
    let node = node_index(&bytes, &p);
    let mut value: Value = serde_json::from_slice(&bytes).expect("json");
    value["semantic_graph"]["nodes"][node]["semantic_type"] = id("scalar_type/integer");
    cases.push((
        "integer operand",
        serde_json::to_vec(&value).expect("bytes"),
    ));

    let mut document = Document::new();
    let p = document.parameter();
    let inner = document.claim(&p);
    let outer = document.claim(&inner);
    cases.push(("reference to a claim", document.bytes(&[&outer])));

    let (document, claim, _) = cycle_document();
    cases.push(("reference cycle", document.bytes(&[&claim])));

    let mut document = Document::new();
    let (claim, _) = not_chain(&mut document, 128);
    cases.push(("depth 129", document.bytes(&[&claim])));

    let mut document = Document::new();
    let mut sized = Sized::new(&mut document);
    let body = sized.node(&mut document, 9_998);
    let last = document.op(NOT, &[&sized.parameter]);
    let top = document.op(AND, &[&body, &last]);
    let claim = document.claim(&top);
    cases.push(("10,001 nodes", document.bytes(&[&claim])));

    let mut document = Document::new();
    let mut sized = Sized::new(&mut document);
    let top = sized.node(&mut document, 10_000);
    let claims = (0..11).map(|_| document.claim(&top)).collect::<Vec<_>>();
    let labels = claims.iter().map(String::as_str).collect::<Vec<_>>();
    cases.push(("100,001 nodes over claims", document.bytes(&labels)));

    for (label, bytes) in cases {
        let fixture = Fixture::new(&bytes);
        let run = fixture.run();
        assert_eq!(run.code, Some(2), "{label}: {}", run.stderr);
        assert!(
            run.stderr.contains("semantic_graph.nodes["),
            "{label}: {}",
            run.stderr
        );
        assert!(fixture.requests().is_empty(), "{label}");
        fixture.assert_no_report();
    }
}

/// FR-009-AC-28: an unremovable capability root is named on stderr; the exit status stays.
#[trace("FR-009-AC-28")]
#[test]
fn an_unremovable_capability_root_is_named_and_keeps_the_exit_status() {
    let built = consistent();
    let fixture = Fixture::new(&built.bytes);
    fixture.both(&built.model, &built.model);
    fs::write(fixture.path("stub/data/undeletable"), "").expect("flag");
    let run = fixture.run();
    let leftovers = directory_names(&fixture.path("out"));
    // Restore access so that the temporary directory can be removed.
    for name in &leftovers {
        let locked = fixture.path("out").join(name).join("locked");
        if locked.exists() {
            fs::set_permissions(&locked, fs::Permissions::from_mode(0o700)).expect("restore");
        }
    }
    assert_eq!(run.code, Some(0), "{}", run.stderr);
    assert!(
        run.stderr.contains("cannot remove the capability root"),
        "{}",
        run.stderr
    );
    assert!(fixture.output().exists());
}

/// A result line past the bound ends the executor and exits `4` without a report.
#[trace("FR-009-AC-5")]
#[test]
fn an_oversize_result_line_exits_four_without_a_report() {
    let built = consistent();
    let fixture = Fixture::new(&built.bytes);
    fs::write(fixture.path("stub/data/z3.version.flood"), "").expect("flag");
    let run = fixture.run();
    assert_eq!(run.code, Some(4), "{}", run.stderr);
    assert!(run.stderr.contains("exceeds"), "{}", run.stderr);
    fixture.assert_no_report();
}

fn find_executable(name: &str) -> Option<PathBuf> {
    let path = std::env::var_os("PATH")?;
    std::env::split_paths(&path)
        .map(|directory| directory.join(name))
        .find(|candidate| candidate.is_file())
        .and_then(|candidate| fs::canonicalize(candidate).ok())
}

fn real_producer_execute() -> Option<PathBuf> {
    find_executable("engineering-assurance")
}

/// Every logged request validates against the schema `producer-execute --schema request` prints.
#[trace("FR-009-AC-15")]
#[test]
fn logged_requests_validate_against_the_published_request_schema() {
    let Some(executable) = real_producer_execute() else {
        eprintln!("skipped: no engineering-assurance executable on PATH");
        return;
    };
    let schema = Command::new(&executable)
        .args(["producer-execute", "--schema", "request"])
        .output()
        .expect("schema");
    assert!(schema.status.success());
    let schema: Value = serde_json::from_slice(&schema.stdout).expect("schema json");
    let compiled = jsonschema::JSONSchema::compile(&schema).expect("compile schema");

    let built = consistent();
    let fixture = Fixture::new(&built.bytes);
    fixture.both(&built.model, &built.model);
    assert_eq!(fixture.run().code, Some(0));
    let requests = fixture.requests();
    assert_eq!(requests.len(), 4);
    for request in &requests {
        if let Err(errors) = compiled.validate(request) {
            let errors = errors.map(|error| error.to_string()).collect::<Vec<_>>();
            panic!("request is invalid: {errors:?}");
        }
    }
}

fn write_solver(path: &Path, version: &str, query_body: &str) {
    let script = format!(
        "#!/bin/sh\ncase \"$1\" in -version|--version) printf '{version}\\n'; exit 0;; esac\n{query_body}\n"
    );
    fs::write(path, script).expect("solver");
    fs::set_permissions(path, fs::Permissions::from_mode(0o700)).expect("executable");
}

/// NFR-003-AC-1: a stalled and a flooding solver run through the real `producer-execute` give a
/// published report with typed non-conclusive outcomes and exit 3.
#[trace("NFR-003-AC-1")]
#[test]
fn a_stalled_and_a_flooding_solver_are_non_conclusive_through_the_real_executor() {
    let Some(executable) = real_producer_execute() else {
        eprintln!("skipped: no engineering-assurance executable on PATH");
        return;
    };
    let built = consistent();
    let fixture = Fixture::new(&built.bytes);
    write_solver(&fixture.z3(), "Z3 version stub", "while :; do :; done");
    write_solver(
        &fixture.cvc5(),
        "cvc5 version stub",
        "while :; do printf 'xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx\\n'; done",
    );
    let run = fixture.run_with(&fixture.with("--producer-execute", executable));
    assert_eq!(run.code, Some(3), "{}", run.stderr);
    let report = fixture.report();
    let bytes = fs::read(fixture.output()).expect("report");
    validate_report_document(&bytes).expect("valid report");
    assert_eq!(
        outcomes(&report),
        ("timed-out".into(), "stdout-limit".into())
    );
    fixture.assert_no_leftovers();
}

/// FR-009-AC-14: the real executor with real Z3 and cvc5.
#[trace("FR-009-AC-14")]
#[test]
fn real_executor_and_real_solvers_give_a_conclusive_report() {
    let (Some(executable), Some(z3), Some(cvc5)) = (
        real_producer_execute(),
        find_executable("z3"),
        find_executable("cvc5"),
    ) else {
        eprintln!("skipped: engineering-assurance, z3 and cvc5 must all be on PATH");
        return;
    };
    let built = consistent();
    let fixture = Fixture::new(&built.bytes);
    let arguments = fixture.with_real_tools(executable, z3, cvc5);
    let run = fixture.run_with(&arguments);
    assert!(
        matches!(run.code, Some(0 | 1)),
        "{:?} {}",
        run.code,
        run.stderr
    );
    let bytes = fs::read(fixture.output()).expect("report");
    validate_report_document(&bytes).expect("valid report");
}

/// A dose bounded by a maximum, and the maximum by a weight, with the model that satisfies it;
/// with `contradiction` the dose is also asserted to exceed the maximum. The weight product is
/// by `1`, the form the IR admits under the reject policy (FR-012).
fn dose_package(contradiction: bool) -> Built {
    let boolean = "scalar_type/boolean";
    let mut document = Document::new();
    let range = document.range();
    let dose = document.parameter_of(&range);
    let max = document.parameter_of(&range);
    let weight = document.parameter_of(&range);
    let one = document.int_literal(&range, 1);
    let scaled = document.int_op("filament.op.integer.mul", &[&one, &weight], &range);
    let within = document.int_op("filament.op.integer.le", &[&dose, &max], boolean);
    let bounded = document.int_op("filament.op.integer.le", &[&max, &scaled], boolean);
    let mut claims = vec![document.claim(&within), document.claim(&bounded)];
    if contradiction {
        let above = document.int_op("filament.op.integer.gt", &[&dose, &max], boolean);
        claims.push(document.claim(&above));
    }
    let labels = claims.iter().map(String::as_str).collect::<Vec<_>>();
    let bytes = document.bytes(&labels);
    let package = match ContractPackageWireV1::read(
        &bytes,
        CheckedPackageReadLimits::bounded(),
        &Default::default(),
    ) {
        ContractPackageReadResult::Admitted(package) => *package,
        other => panic!("fixture refused: {other:?}"),
    };
    let translation = translate_analysis_claims(&package).expect("translation");
    let definitions = translation
        .environment()
        .values()
        .iter()
        .map(|declaration| format!("(define-fun {} () Int 0)", declaration.name().as_str()))
        .collect::<Vec<_>>()
        .join(" ");
    Built {
        model: format!("sat\n(model {definitions})\n"),
        bytes,
    }
}

#[trace("FR-012-AC-1")]
#[test]
fn a_dose_bounded_by_a_maximum_runs_end_to_end() {
    let built = dose_package(false);
    let fixture = Fixture::new(&built.bytes);
    fixture.both(&built.model, &built.model);
    let run = fixture.run();
    assert_eq!(run.code, Some(0), "{}", run.stderr);
    let bytes = fs::read(fixture.output()).expect("report");
    validate_report_document(&bytes).expect("report validates");
    let report: Value = serde_json::from_slice(&bytes).expect("report json");
    assert_eq!(
        report["assertions"].as_array().expect("assertions").len(),
        2,
        "both dose <= max and max <= 1 * weight must reach the report"
    );
    assert_eq!(report["differential"]["agreedStatus"], "satisfied");
}

#[trace("FR-012-AC-2")]
#[test]
fn contradictory_dose_claims_are_refuted() {
    let built = dose_package(true);
    let fixture = Fixture::new(&built.bytes);
    fixture.both("unsat\n", "unsat\n");
    let run = fixture.run();
    assert_eq!(run.code, Some(1), "{}", run.stderr);
    assert_eq!(fixture.report()["differential"]["agreedStatus"], "refuted");
}

/// FR-012-AC-8: the dose fixtures with the real executor and real Z3 and cvc5.
#[trace("FR-012-AC-8")]
#[test]
fn real_solvers_agree_with_the_controlled_run_on_the_dose_fixtures() {
    let executable = real_producer_execute().expect("engineering-assurance is required on PATH");
    let z3 = find_executable("z3").expect("z3 is required on PATH");
    let cvc5 = find_executable("cvc5").expect("cvc5 is required on PATH");
    for (contradiction, code, status) in [(false, 0, "satisfied"), (true, 1, "refuted")] {
        let built = dose_package(contradiction);
        let fixture = Fixture::new(&built.bytes);
        let mut arguments = fixture.with("--producer-execute", executable.clone());
        for (option, value) in [("--z3", z3.clone()), ("--cvc5", cvc5.clone())] {
            let position = arguments.iter().position(|a| a == option).expect("option");
            arguments[position + 1] = value.into();
        }
        let run = fixture.run_with(&arguments);
        assert_eq!(run.code, Some(code), "{}", run.stderr);
        assert_eq!(fixture.report()["differential"]["agreedStatus"], status);
    }
}

/// FR-012-AC-3: the non-linear, division and cycle refusals through the spawned runner.
#[trace("FR-012-AC-3")]
#[test]
fn integer_refusals_exit_two_naming_the_node_through_the_runner() {
    let mut cases: Vec<(Vec<u8>, String)> = Vec::new();

    let (document, claim, product) = product_document();
    cases.push((document.bytes(&[&claim]), product));

    for name in ["div", "rem", "mod"] {
        let (bytes, node) = untranslatable();
        let bytes = with_identity(&bytes, &node, &format!("filament.op.integer.{name}"));
        cases.push((bytes, node));
    }

    let (document, claim, [_, second]) = integer_cycle_document();
    cases.push((document.bytes(&[&claim]), second));

    for (bytes, label) in cases {
        let fixture = Fixture::new(&bytes);
        let run = fixture.run();
        assert_eq!(run.code, Some(2), "{label}: {}", run.stderr);
        let node = format!("semantic_graph.nodes[{}]", node_index(&bytes, &label));
        assert!(
            run.stderr.contains(&node),
            "{label}: {node}: {}",
            run.stderr
        );
        assert!(fixture.requests().is_empty());
        fixture.assert_no_report();
    }
}

/// `role = Admin or role = Auditor` and `role = Guest`: contradictory over the three roles.
fn guest_membership() -> Vec<u8> {
    let mut document = Document::new();
    let role = document.enum_type("Role", &["Admin", "Auditor", "Guest"]);
    let r = document.parameter_of(&role);
    let admin = document.enum_literal(&role, "Admin");
    let auditor = document.enum_literal(&role, "Auditor");
    let guest = document.enum_literal(&role, "Guest");
    let is_admin = document.op(ENUM_EQ, &[&r, &admin]);
    let is_auditor = document.op(ENUM_EQ, &[&r, &auditor]);
    let member = document.op("filament.op.boolean.or", &[&is_admin, &is_auditor]);
    let is_guest = document.op(ENUM_EQ, &[&r, &guest]);
    let first = document.claim(&member);
    let second = document.claim(&is_guest);
    document.bytes(&[&first, &second])
}

#[trace("FR-022-AC-1", "TC-020")]
#[test]
fn a_contradictory_enum_membership_exits_one_through_the_runner() {
    let bytes = guest_membership();
    let fixture = Fixture::new(&bytes);
    fixture.both("unsat\n", "unsat\n");
    let run = fixture.run();
    assert_eq!(run.code, Some(1), "{}", run.stderr);
    assert_eq!(fixture.report()["differential"]["agreedStatus"], "refuted");
    fixture.assert_no_leftovers();
}

/// A satisfiable enum model that omits the datatype symbol fails replay, so it is inconclusive.
#[trace("FR-022-AC-1", "FR-021-AC-5", "TC-020")]
#[test]
fn a_satisfiable_enum_model_without_its_symbol_is_not_verified_and_exits_three() {
    let bytes = guest_membership();
    let fixture = Fixture::new(&bytes);
    fixture.both("sat\n(model)\n", "sat\n(model)\n");
    let run = fixture.run();
    assert_eq!(run.code, Some(3), "{}", run.stderr);
    let report = fixture.report();
    assert_eq!(report["differential"]["disposition"], "inconclusive");
    assert_eq!(report["differential"]["agreedStatus"], Value::Null);
    for engine in report["engines"].as_array().expect("engines") {
        assert_eq!(engine["status"], "satisfied");
        assert_eq!(engine["explanation"], "incomplete");
    }
}

/// Datatype refusals exit `2` naming the node, write no report and invoke no solver.
#[trace("FR-022-AC-3", "FR-022-AC-5", "TC-020")]
#[test]
fn datatype_refusals_exit_two_through_the_runner_without_a_solver() {
    let mut cases: Vec<(Vec<u8>, String)> = Vec::new();

    // A record nesting text.
    let mut document = Document::new();
    let role = document.enum_type("Role", &["Admin", "Guest"]);
    let text = document.unsupported_type("text");
    let user = document.record_type("User", &[("role", &role), ("name", &text)]);
    let u = document.parameter_of(&user);
    let projected = document.project(&user, "role", &u, &role);
    let admin = document.enum_literal(&role, "Admin");
    let is_admin = document.op(ENUM_EQ, &[&projected, &admin]);
    let claim = document.claim(&is_admin);
    cases.push((document.bytes(&[&claim]), text));

    // An identity the table does not hold.
    let mut document = Document::new();
    let role = document.enum_type("Role", &["Admin", "Guest"]);
    let r = document.parameter_of(&role);
    let admin = document.enum_literal(&role, "Admin");
    let is_admin = document.op(ENUM_EQ, &[&r, &admin]);
    let claim = document.claim(&is_admin);
    let bytes = with_identity(
        &document.bytes(&[&claim]),
        &is_admin,
        "filament.op.collection.contains",
    );
    cases.push((bytes, is_admin));

    // A shared enum chain far over the node bound.
    let mut document = Document::new();
    let role = document.enum_type("Role", &["Admin", "Guest"]);
    let r = document.parameter_of(&role);
    let admin = document.enum_literal(&role, "Admin");
    let mut top = document.op(ENUM_EQ, &[&r, &admin]);
    for _ in 0..60 {
        top = document.op("filament.op.boolean.or", &[&top, &top]);
    }
    let claim = document.claim(&top);
    let bytes = document.bytes(&[&claim]);
    for (bytes, label) in cases
        .into_iter()
        .chain(std::iter::once((bytes, String::new())))
    {
        let fixture = Fixture::new(&bytes);
        let started = std::time::Instant::now();
        let run = fixture.run();
        assert_eq!(run.code, Some(2), "{label}: {}", run.stderr);
        if label.is_empty() {
            assert!(
                run.stderr.contains("semantic_graph.nodes["),
                "{}",
                run.stderr
            );
        } else {
            let node = format!("semantic_graph.nodes[{}]", node_index(&bytes, &label));
            assert!(
                run.stderr.contains(&node),
                "{label}: {node}: {}",
                run.stderr
            );
        }
        assert!(started.elapsed() < std::time::Duration::from_secs(30));
        assert!(fixture.requests().is_empty());
        fixture.assert_no_report();
    }
}

/// An unguarded `filament.op.option.value` exits `2` naming the clause and the validator's path.
#[trace("FR-022-AC-3", "TC-020")]
#[test]
fn an_unguarded_option_value_exits_two_naming_the_clause_through_the_runner() {
    let mut document = Document::new();
    let role = document.enum_type("Role", &["Admin", "Guest"]);
    let maybe = document.option_type("MaybeRole", &role);
    let o = document.parameter_of(&maybe);
    let value = document.option_value(&o, &role);
    let admin = document.enum_literal(&role, "Admin");
    let is_admin = document.op(ENUM_EQ, &[&value, &admin]);
    let claim = document.claim(&is_admin);
    let bytes = document.bytes(&[&claim]);
    let fixture = Fixture::new(&bytes);
    let run = fixture.run();
    assert_eq!(run.code, Some(2), "{}", run.stderr);
    let clause = format!("claim-{}", node_index(&bytes, &claim));
    assert!(run.stderr.contains(&clause), "{clause}: {}", run.stderr);
    let expected = format!("{clause} at expression: option presence is not established");
    assert!(run.stderr.contains(&expected), "{expected}: {}", run.stderr);
    assert!(fixture.requests().is_empty());
    fixture.assert_no_report();
}

/// FR-022-AC-1 fixtures 3 to 5 need the record and option lowering; a satisfiable answer whose
/// model omits the datatype symbol fails replay and exits `3`.
#[trace("FR-022-AC-1", "TC-020")]
#[test]
fn record_and_option_fixtures_have_their_fixed_exits_through_the_runner() {
    let satisfiable = "sat\n(model)\n";
    let cases: [(Vec<u8>, &str, i32); 3] = [
        (bytes_of(record_field(false)), satisfiable, 3),
        (bytes_of(record_field(true)), "unsat\n", 1),
        (
            {
                let (document, claims, _) = option_value(true);
                document.bytes(&labels(&claims))
            },
            satisfiable,
            3,
        ),
    ];
    for (bytes, answer, code) in cases {
        let fixture = Fixture::new(&bytes);
        fixture.both(answer, answer);
        let run = fixture.run();
        assert_eq!(run.code, Some(code), "{}", run.stderr);
    }
}

fn bytes_of((document, claims): (Document, Vec<String>)) -> Vec<u8> {
    document.bytes(&labels(&claims))
}

// ---- TC-006 retained run-package corpus (FR-009-AC-38, FR-009-AC-39) ----

/// The report bytes with the number after every `"elapsedMs":` replaced by `0`, byte for byte
/// otherwise.
fn zero_elapsed_bytes(bytes: &[u8]) -> Vec<u8> {
    const KEY: &[u8] = b"\"elapsedMs\"";
    let mut out = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index..].starts_with(KEY) {
            out.extend_from_slice(KEY);
            index += KEY.len();
            while bytes.get(index).is_some_and(|b| matches!(b, b':' | b' ')) {
                out.push(bytes[index]);
                index += 1;
            }
            while bytes.get(index).is_some_and(u8::is_ascii_digit) {
                index += 1;
            }
            out.push(b'0');
        } else {
            out.push(bytes[index]);
            index += 1;
        }
    }
    out
}

/// What one corpus case gave through both boundaries, which have already been compared.
struct Corpus {
    code: u8,
    /// The report the library run published, with its `elapsedMs` values as measured.
    report: Option<Vec<u8>>,
    /// The library error, when the library run published no report.
    error: Option<filament_analyze::RunnerError>,
    stderr: String,
}

/// The report at the output path, removed so the next run may publish there.
fn take_report(fixture: &Fixture) -> Option<Vec<u8>> {
    let bytes = fs::read(fixture.output()).ok()?;
    fs::remove_file(fixture.output()).expect("remove report");
    Some(bytes)
}

/// Run `fixture` through `run_package` and through the spawned binary with one argument vector,
/// and require the same exit status and, once every `elapsedMs` is `0`, the same report bytes.
fn through_both(fixture: &Fixture) -> Corpus {
    let arguments = fixture.arguments();
    let parsed = filament_analyze::parse_run_package_arguments(arguments[1..].iter().cloned());
    let Ok(filament_analyze::RunPackageCommand::Run(parsed)) = parsed else {
        panic!("the shared argument vector does not parse: {parsed:?}");
    };
    let library = filament_analyze::run_package(&parsed);
    let library_report = take_report(fixture);
    let binary = fixture.run_with(&arguments);
    let binary_report = take_report(fixture);

    let (code, error) = match library {
        Ok(code) => (code, None),
        Err(error) => (error.class(), Some(error)),
    };
    assert_eq!(
        binary.code,
        Some(i32::from(code)),
        "the boundaries disagree on the exit status: {}",
        binary.stderr
    );
    assert_eq!(
        library_report.as_deref().map(zero_elapsed_bytes),
        binary_report.as_deref().map(zero_elapsed_bytes),
        "the boundaries disagree on the report"
    );
    if let Some(error) = &error {
        assert!(library_report.is_none(), "a failed run published a report");
        assert!(
            binary.stderr.contains(error.message()),
            "stderr {:?} omits {:?}",
            binary.stderr,
            error.message()
        );
    }
    fixture.assert_no_leftovers();
    Corpus {
        code,
        report: library_report,
        error,
        stderr: binary.stderr,
    }
}

/// A published report that validates, parsed.
fn valid_report(corpus: &Corpus) -> Value {
    let bytes = corpus.report.as_ref().expect("a report was published");
    validate_report_document(bytes).expect("valid report");
    serde_json::from_slice(bytes).expect("report json")
}

/// A case that is not conclusive: both engine records retained, the stated disposition and
/// outcome of the first engine, and no agreed status.
fn assert_non_conclusive(corpus: &Corpus, disposition: &str, outcome: (&str, &str)) {
    assert_eq!(corpus.code, 3, "{}", corpus.stderr);
    let report = valid_report(corpus);
    assert_eq!(report["engines"].as_array().expect("engines").len(), 2);
    assert_eq!(report["differential"]["disposition"], disposition);
    assert_eq!(outcomes(&report), (outcome.0.into(), outcome.1.into()));
    assert_eq!(report["differential"]["agreedStatus"], Value::Null);
}

/// The same package as a case, with healthy answers: the case's stub result is the only change.
fn assert_healthy_control(built: &Built) {
    let fixture = Fixture::new(&built.bytes);
    fixture.both(&built.model, &built.model);
    let corpus = through_both(&fixture);
    assert_eq!(corpus.code, 0, "{}", corpus.stderr);
    assert_eq!(
        valid_report(&corpus)["differential"]["agreedStatus"],
        "satisfied"
    );
}

#[trace("FR-009-AC-38")]
#[test]
fn rp_sat_exits_zero_through_both_boundaries() {
    let built = consistent();
    let fixture = Fixture::new(&built.bytes);
    fixture.both(&built.model, &built.model);
    let corpus = through_both(&fixture);
    assert_eq!(corpus.code, 0, "{}", corpus.stderr);
    let report = valid_report(&corpus);
    assert_eq!(report["differential"]["disposition"], "agreement");
    assert_eq!(report["differential"]["agreedStatus"], "satisfied");

    // Control: one engine answering `unsat` is not agreement, so exit `0` rests on agreement.
    let control = Fixture::new(&built.bytes);
    control.both(&built.model, "unsat\n");
    assert_eq!(through_both(&control).code, 3);
}

#[trace("FR-009-AC-38")]
#[test]
fn rp_unsat_exits_one_through_both_boundaries() {
    let built = contradictory();
    let fixture = Fixture::new(&built.bytes);
    fixture.both("unsat\n", "unsat\n");
    let corpus = through_both(&fixture);
    assert_eq!(corpus.code, 1, "{}", corpus.stderr);
    let report = valid_report(&corpus);
    assert_eq!(report["differential"]["disposition"], "agreement");
    assert_eq!(report["differential"]["agreedStatus"], "refuted");

    // Control: the same package answered `sat` by both engines is not refuted.
    let control = Fixture::new(&built.bytes);
    control.both(&built.model, &built.model);
    let corpus = through_both(&control);
    assert_ne!(corpus.code, 1, "{}", corpus.stderr);
}

#[trace("FR-009-AC-38")]
#[test]
fn rp_unsupported_exits_two_naming_the_node_without_a_report_or_request() {
    let (bytes, quotient) = untranslatable();
    let fixture = Fixture::new(&bytes);
    let corpus = through_both(&fixture);
    assert_eq!(corpus.code, 2, "{}", corpus.stderr);
    assert!(corpus.report.is_none());
    let node = format!("semantic_graph.nodes[{}]", node_index(&bytes, &quotient));
    let error = corpus.error.as_ref().expect("a library error");
    assert!(error.message().contains(&node), "{node}: {error}");
    assert!(corpus.stderr.contains(&node), "{node}: {}", corpus.stderr);
    assert!(fixture.requests().is_empty(), "producer-execute was called");
    fixture.assert_no_report();

    // Control: a translatable package under the same vector does reach `producer-execute`.
    let built = consistent();
    let control = Fixture::new(&built.bytes);
    control.both(&built.model, &built.model);
    assert_eq!(through_both(&control).code, 0);
    assert_eq!(control.requests().len(), 8, "two runs of four requests");
}

#[trace("FR-009-AC-38")]
#[test]
fn rp_timeout_is_inconclusive_through_both_boundaries() {
    let built = consistent();
    let fixture = Fixture::new(&built.bytes);
    fixture.both(&built.model, &built.model);
    fixture.answer("z3", "query", &state_only(json!({"kind": "timed_out"})));
    let corpus = through_both(&fixture);
    assert_non_conclusive(&corpus, "inconclusive", ("timed-out", "sat"));
    assert_healthy_control(&built);
}

#[trace("FR-009-AC-38")]
#[test]
fn rp_missing_engine_is_unavailable_through_both_boundaries() {
    let built = consistent();
    let fixture = Fixture::new(&built.bytes);
    fixture.both(&built.model, &built.model);
    fixture.answer("z3", "query", &state_only(json!({"kind": "unavailable"})));
    let corpus = through_both(&fixture);
    assert_non_conclusive(&corpus, "unavailable", ("missing-executable", "sat"));
    assert_healthy_control(&built);
}

#[trace("FR-009-AC-38")]
#[test]
fn rp_disagree_is_a_disagreement_through_both_boundaries() {
    let built = consistent();
    let fixture = Fixture::new(&built.bytes);
    fixture.both(&built.model, "unsat\n");
    let corpus = through_both(&fixture);
    assert_non_conclusive(&corpus, "disagreement", ("sat", "unsat"));

    // The same with the engines swapped: no engine's answer settles the disagreement.
    let swapped = Fixture::new(&built.bytes);
    swapped.both("unsat\n", &built.model);
    let corpus = through_both(&swapped);
    assert_non_conclusive(&corpus, "disagreement", ("unsat", "sat"));
    assert_healthy_control(&built);
}

/// The real executable, Z3 and cvc5, or the reason one is absent.
fn real_tools() -> Result<(PathBuf, PathBuf, PathBuf), String> {
    let find = |name: &str| find_executable(name).ok_or_else(|| format!("{name} is not on PATH"));
    Ok((find("engineering-assurance")?, find("z3")?, find("cvc5")?))
}

/// FR-009-AC-39: with the real tools, `built` exits `code` with `status`, and the report validates.
fn real_case(built: &Built, code: i32, status: &str) {
    let (executable, z3, cvc5) = match real_tools() {
        Ok(tools) => tools,
        Err(reason) => {
            eprintln!("skipped, not evidence for FR-009-AC-38: {reason}");
            return;
        }
    };
    let fixture = Fixture::new(&built.bytes);
    let arguments = fixture.with_real_tools(executable, z3, cvc5);
    let run = fixture.run_with(&arguments);
    assert_eq!(run.code, Some(code), "{}", run.stderr);
    let bytes = fs::read(fixture.output()).expect("report");
    validate_report_document(&bytes).expect("valid report");
    assert_eq!(fixture.report()["differential"]["agreedStatus"], status);
}

#[trace("FR-009-AC-39")]
#[test]
fn rp_sat_with_real_engines_is_satisfied() {
    real_case(&consistent(), 0, "satisfied");
}

#[trace("FR-009-AC-39")]
#[test]
fn rp_unsat_with_real_engines_is_refuted() {
    real_case(&contradictory(), 1, "refuted");
}
