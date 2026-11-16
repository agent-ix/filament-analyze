//! The unix execution half of the `run-package` runner (FR-009): capability root, staged
//! query, `producer-execute` invocation and publication.

use std::{
    collections::{BTreeMap, BTreeSet},
    fmt, fs,
    io::{self, Read, Write},
    os::unix::fs::{DirBuilderExt, OpenOptionsExt},
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use filament_contract_model::{
    CheckedNodeId, CheckedPackageReadLimits, CheckedTerm, ContractPackageReadResult,
    ContractPackageV1, ContractPackageWireV1,
};
use serde::Deserialize;

use super::{
    publish_with, RunPackageArguments, RunPackageRequest, RunnerError, EXIT_NON_CONCLUSIVE,
    EXIT_REFUTED, EXIT_SATISFIED,
};
use crate::{
    analysis_claim_indices, compare_solver_records, invoke_producer_execute,
    lower_analysis_request, publish_report_new, query_request,
    render_differential_report_with_selection, render_differential_summary,
    solver_record_from_evidence, translate_analysis_claims_for_request,
    validate_differential_report_with_selection, version_request, version_text, AdapterLimits,
    AnalysisStatus, ClaimSelectionReport, DifferentialDisposition, ProducerExecutionRequest,
    ProducerExecutionResult, QueryBundle, SolverConfig, SolverEngine, SolverRecord,
    MAX_SOLVER_PATH_BYTES, MAX_SOLVER_WALL_TIME_MS,
};

/// The staged query file inside the capability root.
const QUERY_FILE: &str = "query.smt2";
/// The most bytes of the package file the runner reads, one past the reader's ceiling so that
/// an oversized document is refused by the reader as `Incomplete`.
const MAX_PACKAGE_READ_BYTES: u64 = CheckedPackageReadLimits::bounded().bytes + 1;
const MAX_SELECTION_BYTES: u64 = 1024 * 1024;
const SELECTION_PROTOCOL: &str = "filament-analyze.claim-selection/v1";

static ROOT_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Run the analysis and return the exit status of a published report.
///
/// # Errors
///
/// Returns the exit status and diagnostic of a usage, input, executor or publication failure;
/// no report is written for such a failure, except that a publication that fails with
/// `published-durability-unknown` leaves the renamed report in place.
pub fn run_package(arguments: &RunPackageArguments) -> Result<u8, RunnerError> {
    let producer = checked_executable(&arguments.producer_execute, "--producer-execute")?;
    let z3 = solver_config(SolverEngine::Z3, &arguments.z3, "--z3")?;
    let cvc5 = solver_config(SolverEngine::Cvc5, &arguments.cvc5, "--cvc5")?;
    let output = std::path::absolute(&arguments.output).map_err(|error| {
        RunnerError::usage(format!(
            "--output {}: cannot be resolved: {error}",
            arguments.output.display()
        ))
    })?;
    let parent = output.parent().ok_or_else(|| {
        RunnerError::usage(format!("--output {}: has no parent", output.display()))
    })?;
    reject_symlinks(parent, "--output parent")?;

    let (query, selection) = read_query(
        &arguments.package,
        arguments.select.as_deref(),
        arguments.request,
    )?;

    if fs::symlink_metadata(&output).is_ok() {
        return Err(RunnerError::failure(format!(
            "--output {}: the destination exists",
            output.display()
        )));
    }
    let root = CapabilityRoot::create(parent).map_err(|error| {
        RunnerError::failure(format!(
            "cannot create the capability root under {}: {error}",
            parent.display()
        ))
    })?;
    root.stage_query(query.query().as_bytes())
        .map_err(|error| RunnerError::failure(format!("cannot stage the query: {error}")))?;

    let z3_record = run_engine(&producer, root.path(), &query, &z3)?;
    let cvc5_record = run_engine(&producer, root.path(), &query, &cvc5)?;
    let run = compare_solver_records(&query, &z3_record, &cvc5_record);
    let bytes = render_differential_report_with_selection(&query, &run, selection.as_ref())
        .and_then(|bytes| {
            validate_differential_report_with_selection(&bytes, &query, &run, selection.as_ref())
                .map(|()| bytes)
        })
        .map_err(|error| RunnerError::failure(format!("the report is not valid: {error}")))?;
    publish_with(&output, &bytes, publish_report_new)?;
    drop(root);
    eprint!("{}", render_differential_summary(&run));
    eprintln!("filament-analyze: published {}", output.display());
    Ok(exit_status(run.disposition(), run.agreed_status()))
}

fn exit_status(disposition: DifferentialDisposition, agreed: Option<AnalysisStatus>) -> u8 {
    match (disposition, agreed) {
        (DifferentialDisposition::Agreement, Some(AnalysisStatus::Satisfied)) => EXIT_SATISFIED,
        (DifferentialDisposition::Agreement, Some(AnalysisStatus::Refuted)) => EXIT_REFUTED,
        _ => EXIT_NON_CONCLUSIVE,
    }
}

/// An absolute, UTF-8, bounded path without a NUL and without a symbolic link in any component.
fn checked_executable(path: &Path, label: &str) -> Result<PathBuf, RunnerError> {
    let text = path
        .to_str()
        .ok_or_else(|| RunnerError::usage(format!("{label}: the path is not UTF-8")))?;
    if text.is_empty() || !path.is_absolute() {
        return Err(RunnerError::usage(format!(
            "{label} {text:?}: the path must be absolute and non-empty"
        )));
    }
    if text.contains('\0') || text.len() > MAX_SOLVER_PATH_BYTES {
        return Err(RunnerError::usage(format!(
            "{label}: the path contains NUL or exceeds {MAX_SOLVER_PATH_BYTES} bytes"
        )));
    }
    reject_symlinks(path, label)?;
    Ok(path.to_path_buf())
}

fn solver_config(
    engine: SolverEngine,
    path: &Path,
    label: &str,
) -> Result<SolverConfig, RunnerError> {
    let path = checked_executable(path, label)?;
    SolverConfig::new(engine, path, AdapterLimits::default())
        .map_err(|error| RunnerError::usage(format!("{label}: {}", error.message())))
}

/// Refuse a path with a symbolic link in any component. A component that does not exist is not
/// a link, and nothing below it exists.
fn reject_symlinks(path: &Path, label: &str) -> Result<(), RunnerError> {
    let mut prefix = PathBuf::new();
    for component in path.components() {
        prefix.push(component);
        match fs::symlink_metadata(&prefix) {
            Ok(metadata) if metadata.file_type().is_symlink() => {
                return Err(RunnerError::usage(format!(
                    "{label} {}: {} is a symbolic link",
                    path.display(),
                    prefix.display()
                )));
            }
            Ok(_) => {}
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(()),
            Err(error) => {
                return Err(RunnerError::usage(format!(
                    "{label} {}: cannot inspect {}: {error}",
                    path.display(),
                    prefix.display()
                )));
            }
        }
    }
    Ok(())
}

/// The source spans that identify draft declarations under review.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ClaimSelectionDocument {
    protocol: String,
    spans: Vec<ClaimSelectionSpan>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ClaimSelectionSpan {
    authority: String,
    identity: String,
    start: u64,
    end: u64,
}

/// Read and admit the package, apply any source selection, translate it and lower the request.
fn read_query(
    package: &Path,
    selection_path: Option<&Path>,
    requested: RunPackageRequest,
) -> Result<(QueryBundle, Option<ClaimSelectionReport>), RunnerError> {
    let unreadable = |error: &dyn fmt::Display| {
        RunnerError::usage(format!("--package {}: {error}", package.display()))
    };
    let metadata = fs::metadata(package).map_err(|error| unreadable(&error))?;
    if !metadata.is_file() {
        return Err(unreadable(&"not a regular file"));
    }
    let mut bytes = Vec::new();
    fs::File::open(package)
        .and_then(|file| file.take(MAX_PACKAGE_READ_BYTES).read_to_end(&mut bytes))
        .map_err(|error| unreadable(&error))?;
    let admitted = match ContractPackageWireV1::read(
        &bytes,
        CheckedPackageReadLimits::bounded(),
        &Default::default(),
    ) {
        ContractPackageReadResult::Admitted(admitted) => admitted,
        ContractPackageReadResult::Refused(refusal) => {
            return Err(unreadable(&format!("the package is refused: {refusal}")));
        }
        ContractPackageReadResult::Incomplete(incomplete) => {
            return Err(unreadable(&format!(
                "the package exhausts a read limit: {incomplete:?}"
            )));
        }
        ContractPackageReadResult::CatalogUnavailable(error) => {
            return Err(RunnerError::failure(format!(
                "the package reader's operation catalog is unavailable: {error}"
            )));
        }
    };
    let selected = selection_path
        .map(|path| read_claim_selection(path, &admitted))
        .transpose()?;
    let selected_indices = selected.as_ref().map(|(indices, _)| indices);
    let translation = translate_analysis_claims_for_request(&admitted, selected_indices, requested)
        .map_err(|error| RunnerError::usage(format!("--package: {error}")))?;
    let query = lower_analysis_request(translation.request()).map_err(|errors| {
        let cause = errors.first().map_or_else(
            || "the request is refused".to_owned(),
            |error| format!("{}: {}", error.path(), error.message()),
        );
        RunnerError::usage(format!("--package: the request cannot be lowered: {cause}"))
    })?;
    let report = selected.map(|(indices, all_claims)| {
        let selected_ids = all_claims
            .iter()
            .filter(|node| indices.contains(node))
            .map(|node| format!("claim-{node}"))
            .collect();
        let excluded_ids = all_claims
            .iter()
            .filter(|node| !indices.contains(node))
            .map(|node| format!("claim-{node}"))
            .collect();
        ClaimSelectionReport::new(selected_ids, excluded_ids)
    });
    Ok((query, report))
}

fn read_claim_selection(
    path: &Path,
    package: &ContractPackageV1,
) -> Result<(BTreeSet<usize>, Vec<usize>), RunnerError> {
    reject_symlinks(path, "--select")?;
    let bytes = read_bounded_file(path, MAX_SELECTION_BYTES, "--select")?;
    let document: ClaimSelectionDocument = serde_json::from_slice(&bytes)
        .map_err(|error| RunnerError::usage(format!("--select {}: {error}", path.display())))?;
    if document.protocol != SELECTION_PROTOCOL {
        return Err(RunnerError::usage(format!(
            "--select {}: protocol must be {SELECTION_PROTOCOL}",
            path.display()
        )));
    }
    if document.spans.is_empty() {
        return Err(RunnerError::usage(format!(
            "--select {}: empty selection",
            path.display()
        )));
    }

    let all_claims = analysis_claim_indices(package);
    let wire = package.wire();
    let mut selected = BTreeSet::new();
    for span in &document.spans {
        validate_selection_span(span)?;
    }

    let mut matched_spans = vec![false; document.spans.len()];
    for &claim in &all_claims {
        let Some(region) = source_region(wire, claim) else {
            continue;
        };
        for (span_index, span) in document.spans.iter().enumerate() {
            if same_source(region, span) && overlaps(region.start, region.end, span.start, span.end)
            {
                if contains(span, region) {
                    selected.insert(claim);
                    matched_spans[span_index] = true;
                } else {
                    return Err(RunnerError::usage(format!(
                        "--select span {} overlaps claim-{claim} without containing it",
                        span.display()
                    )));
                }
            }
        }
    }

    let mut reached_by = BTreeMap::<usize, usize>::new();
    for &claim in &selected {
        let mut pending = Vec::new();
        let mut visited = BTreeSet::new();
        let node_id = wire
            .semantic_graph
            .nodes
            .get(claim)
            .map(|node| &node.node_id);
        let term = node_id.and_then(|id| package.term(id));
        collect_claim_references(node_id, term, &mut pending);
        while let Some(target) = pending.pop() {
            let reached = node_index(wire, &target);
            if !record_reached_node(reached, claim, &mut visited, &mut reached_by) {
                continue;
            }
            if let Some(term) = package.term(&target) {
                collect_references(term, &mut pending);
            }
        }
    }

    for (&node, &claim) in &reached_by {
        let Some(region) = source_region(wire, node) else {
            return Err(RunnerError::usage(format!(
                "--select: claim-{claim} reaches semantic_graph.nodes[{node}] with no source region"
            )));
        };
        let mut covered = false;
        for (span_index, span) in document.spans.iter().enumerate() {
            if contains(span, region) {
                matched_spans[span_index] = true;
                covered = true;
            }
        }
        if !covered {
            return Err(RunnerError::usage(format!(
                "--select: claim-{claim} reaches semantic_graph.nodes[{node}] outside listed spans"
            )));
        }
    }

    if let Some((index, _)) = matched_spans
        .iter()
        .enumerate()
        .find(|(_, matched)| !**matched)
    {
        return Err(RunnerError::usage(format!(
            "--select span {} matches no claim or reached node",
            document.spans[index].display()
        )));
    }
    Ok((selected, all_claims))
}

fn validate_selection_span(span: &ClaimSelectionSpan) -> Result<(), RunnerError> {
    if span.authority.is_empty() || span.identity.is_empty() {
        return Err(RunnerError::usage(format!(
            "--select span {}: authority and identity must be non-empty",
            span.display()
        )));
    }
    if span.start >= span.end {
        return Err(RunnerError::usage(format!(
            "--select span {}: start must be below end",
            span.display()
        )));
    }
    Ok(())
}

impl ClaimSelectionSpan {
    fn display(&self) -> String {
        format!(
            "{}:{}:{}-{}",
            self.authority, self.identity, self.start, self.end
        )
    }
}

fn source_region(
    wire: &ContractPackageWireV1,
    node: usize,
) -> Option<&filament_contract_model::CheckedSourceRegion> {
    wire.source_map
        .nodes
        .get(node)?
        .occurrences
        .first()?
        .regions
        .first()
}

fn same_source(
    region: &filament_contract_model::CheckedSourceRegion,
    span: &ClaimSelectionSpan,
) -> bool {
    region.source.authority.as_ref() == span.authority
        && region.source.identity.as_ref() == span.identity
}

fn contains(
    span: &ClaimSelectionSpan,
    region: &filament_contract_model::CheckedSourceRegion,
) -> bool {
    same_source(region, span) && span.start <= region.start && region.end <= span.end
}

fn overlaps(left_start: u64, left_end: u64, right_start: u64, right_end: u64) -> bool {
    left_start.max(right_start) < left_end.min(right_end)
}

fn node_index(wire: &ContractPackageWireV1, target: &CheckedNodeId) -> Option<usize> {
    wire.semantic_graph
        .nodes
        .iter()
        .position(|node| &node.node_id == target)
}

fn collect_claim_references(
    node_id: Option<&CheckedNodeId>,
    term: Option<&CheckedTerm>,
    references: &mut Vec<CheckedNodeId>,
) {
    let Some(_node_id) = node_id else {
        return;
    };
    let Some(term) = term else {
        return;
    };
    collect_references(term, references);
}

fn record_reached_node(
    node: Option<usize>,
    claim: usize,
    visited: &mut BTreeSet<usize>,
    reached_by: &mut BTreeMap<usize, usize>,
) -> bool {
    let Some(node) = node else {
        return false;
    };
    if !visited.insert(node) {
        return false;
    }
    reached_by.entry(node).or_insert(claim);
    true
}

fn collect_references(term: &CheckedTerm, references: &mut Vec<CheckedNodeId>) {
    match term {
        CheckedTerm::Reference { target } => references.push(target.clone()),
        CheckedTerm::Application { arguments, .. } => {
            for argument in arguments {
                collect_references(argument, references);
            }
        }
        _ => {}
    }
}

fn read_bounded_file(path: &Path, limit: u64, label: &str) -> Result<Vec<u8>, RunnerError> {
    let metadata = fs::metadata(path)
        .map_err(|error| RunnerError::usage(format!("{label} {}: {error}", path.display())))?;
    if !metadata.is_file() {
        return Err(RunnerError::usage(format!(
            "{label} {}: not a regular file",
            path.display()
        )));
    }
    if metadata.len() > limit {
        return Err(RunnerError::usage(format!(
            "{label} {}: exceeds the {limit}-byte limit",
            path.display()
        )));
    }
    let file = fs::File::open(path)
        .map_err(|error| RunnerError::usage(format!("{label} {}: {error}", path.display())))?;
    read_bounded_reader(file, path, limit, label)
}

fn read_bounded_reader(
    reader: impl Read,
    path: &Path,
    limit: u64,
    label: &str,
) -> Result<Vec<u8>, RunnerError> {
    let mut bytes = Vec::new();
    reader
        .take(limit + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| RunnerError::usage(format!("{label} {}: {error}", path.display())))?;
    if u64::try_from(bytes.len()).map_or(true, |size| size > limit) {
        return Err(RunnerError::usage(format!(
            "{label} {}: exceeds the {limit}-byte limit",
            path.display()
        )));
    }
    Ok(bytes)
}

/// The runner-owned capability root, a `0700` directory removed when the guard drops.
struct CapabilityRoot {
    path: PathBuf,
}

impl CapabilityRoot {
    fn create(parent: &Path) -> io::Result<Self> {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |elapsed| elapsed.as_nanos());
        let counter = ROOT_COUNTER.fetch_add(1, Ordering::Relaxed);
        let path = parent.join(format!(
            ".filament-analyze-run-{}-{nanos}-{counter}",
            std::process::id()
        ));
        fs::DirBuilder::new().mode(0o700).create(&path)?;
        Ok(Self { path })
    }

    fn path(&self) -> &Path {
        &self.path
    }

    fn stage_query(&self, query: &[u8]) -> io::Result<()> {
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(self.path.join(QUERY_FILE))?;
        file.write_all(query)?;
        file.sync_all()
    }
}

/// Removal runs on every exit path. A removal that fails is named on stderr and does not change
/// the exit status (FR-009-AC-28).
impl Drop for CapabilityRoot {
    fn drop(&mut self) {
        if let Err(error) = fs::remove_dir_all(&self.path) {
            if error.kind() != io::ErrorKind::NotFound {
                eprintln!(
                    "filament-analyze: cannot remove the capability root {}: {error}",
                    self.path.display()
                );
            }
        }
    }
}

/// Run the version then the query invocation of one engine and build its record.
fn run_engine(
    producer: &Path,
    root: &Path,
    query: &QueryBundle,
    config: &SolverConfig,
) -> Result<SolverRecord, RunnerError> {
    let cause = |error: &dyn fmt::Display| {
        RunnerError::failure(format!("{} request: {error}", config.engine().as_str()))
    };
    let version_request = version_request(config, root).map_err(|error| cause(&error))?;
    let version = invoke(producer, root, &version_request)?;
    let query_result = match version_text(config, &version) {
        Ok(text) => {
            let request = query_request(config, root, &text, Path::new(QUERY_FILE))
                .map_err(|error| cause(&error))?;
            Some(invoke(producer, root, &request)?)
        }
        Err(_) => None,
    };
    Ok(solver_record_from_evidence(
        query,
        config,
        &version,
        query_result.as_ref(),
    ))
}

/// One `producer-execute --request -` process: the request on stdin, exactly one result line on
/// stdout, and a zero exit status.
fn invoke(
    producer: &Path,
    root: &Path,
    request: &ProducerExecutionRequest,
) -> Result<ProducerExecutionResult, RunnerError> {
    let failure = |message: String| RunnerError::failure(format!("producer-execute: {message}"));
    invoke_producer_execute(
        producer,
        root,
        &request.to_json_bytes(),
        Duration::from_millis(MAX_SOLVER_WALL_TIME_MS + 5_000),
        Duration::from_millis(1_000),
    )
    .map_err(|error| failure(error.to_string()))
}

#[cfg(test)]
mod tests {
    use super::{collect_claim_references, read_bounded_reader, record_reached_node};
    use filament_contract_model::{CheckedNodeId, CheckedTerm};
    use serde_json::json;
    use std::collections::{BTreeMap, BTreeSet};
    use std::io::Cursor;

    #[test]
    fn missing_selection_graph_nodes_and_terms_are_ignored_safely() {
        let missing_node: Option<&CheckedNodeId> = None;
        let missing_term: Option<&CheckedTerm> = None;
        let mut references = Vec::new();
        collect_claim_references(missing_node, missing_term, &mut references);

        let node_id = serde_json::from_value::<CheckedNodeId>(json!({
            "package": "test/pkg",
            "path": ["missing-term"],
            "ordinal": 0
        }))
        .expect("well-formed checked node id");
        collect_claim_references(Some(&node_id), missing_term, &mut references);
        assert!(references.is_empty());

        let mut visited = BTreeSet::new();
        let mut reached_by = BTreeMap::new();
        assert!(!record_reached_node(None, 3, &mut visited, &mut reached_by));
        assert!(visited.is_empty());
        assert!(reached_by.is_empty());
    }

    #[test]
    fn bounded_reader_rejects_a_stream_that_grows_past_the_metadata_limit() {
        let error = read_bounded_reader(
            Cursor::new(b"12345"),
            std::path::Path::new("input"),
            4,
            "--select",
        )
        .expect_err("stream exceeded limit after metadata check");
        assert_eq!(error.class(), super::super::EXIT_USAGE);
        assert!(error.message().contains("exceeds the 4-byte limit"));
    }
}
