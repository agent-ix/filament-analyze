//! FR-009 translation of `analysis_claim` nodes into analysis statements (TC-015 case 19).
//!
//! Every contract package here is built at test time through the public path:
//! a `filament.checked-package/v2` document built in this file, `CheckedPackageV2::read`,
//! `CheckedPackageV2::lower` with `CompleteLoweringProfileV2`, the canonical bytes of the
//! `CompleteContractPackageV2`, and `ContractPackageWireV1::read`.

use filament_analyze::{
    lower_analysis_request, translate_analysis_claims, translate_analysis_claims_for_request,
    RunPackageRequest, TranslationError, TranslationErrorKind,
};
use filament_contract_model::{
    BooleanOperator, CheckedPackageReadLimits, ComparisonOperator, ContractPackageReadResult,
    ContractPackageV1, ContractPackageWireV1, ExecutionPoint, ExpressionKind, ValueDeclarationKind,
    QuantifierKind, ValueType,
};
use ix_trace_rs::trace;
use serde_json::{json, Value};
use std::collections::BTreeSet;

mod common;

use common::*;

#[trace("TC-030", "FR-034-AC-1")]
#[test]
fn package_quantifiers_translate_from_literal_collection_domains() {
    for kind in ["set", "ordered_set", "sequence", "bag"] {
        let mut document = Document::new();
        let binder = document.parameter();
        let first = document.literal(true);
        let second = document.literal(false);
        let collection = document.collection_literal(
            kind,
            "scalar_type/boolean",
            &[&first, &second],
        );
        assert_both_quantifier_forms(&mut document, &collection, "p1", &binder);
    }

    let mut document = Document::new();
    let color = document.enum_type("Color", &["red", "blue"]);
    let binder = document.parameter_of(&color);
    let red = document.enum_literal(&color, "red");
    let blue = document.enum_literal(&color, "blue");
    let is_red = document.op(ENUM_EQ, &[&binder, &red]);
    let collection = document.collection_literal("sequence", &color, &[&red, &blue]);
    assert_both_quantifier_forms(&mut document, &collection, "p1", &is_red);

    let mut document = Document::new();
    let range = document.range_between("quantified", 0, 3);
    let binder = document.parameter_of(&range);
    let one = document.int_literal(&range, 1);
    let two = document.int_literal(&range, 2);
    let is_one = document.int_op("filament.op.integer.eq", &[&binder, &one], "scalar_type/boolean");
    let collection = document.collection_literal("bag", &range, &[&one, &two]);
    assert_both_quantifier_forms(&mut document, &collection, "p1", &is_one);
}

fn assert_both_quantifier_forms(document: &mut Document, collection: &str, binder: &str, body: &str) {
    let all = document.quantifier("filament.op.collection.forall", collection, binder, body);
    let any = document.quantifier("filament.op.collection.exists", collection, binder, body);
    let all_claim = document.claim(&all);
    let any_claim = document.claim(&any);
    let package = document.package(&[&all_claim, &any_claim]);
    let translated = translate_analysis_claims(&package).expect("quantifiers translate");

    for position in 0..2 {
        assert!(matches!(
            expression(&translated, position).kind(),
            ExpressionKind::Quantifier { .. }
        ));
    }
    assert_eq!(
        expression(&translated, 0).source().start().byte_offset(),
        document.region(&all).0
    );
    assert_eq!(
        expression(&translated, 1).source().start().byte_offset(),
        document.region(&any).0
    );
    let query = lower_analysis_request(translated.request()).expect("bounded queries lower");
    assert_eq!(query.quantifiers().len(), 2);
    assert_eq!(query.quantifiers()[0].kind, QuantifierKind::ForAll);
    assert_eq!(query.quantifiers()[1].kind, QuantifierKind::Exists);
    assert!(query
        .quantifiers()
        .iter()
        .all(|entry| entry.source.starts_with("semantic_graph.nodes[")));
}

#[trace("TC-030", "FR-034-AC-3")]
#[test]
fn package_quantifiers_reusing_a_variable_name_get_distinct_locals() {
    let mut document = Document::new();
    let binder = document.parameter();
    let first = document.literal(true);
    let second = document.literal(false);
    let outer_domain = document.collection_literal(
        "sequence",
        "scalar_type/boolean",
        &[&first, &second],
    );
    let inner_domain = document.collection_literal(
        "set",
        "scalar_type/boolean",
        &[&first, &second],
    );
    let inner = document.quantifier(
        "filament.op.collection.exists",
        &inner_domain,
        "p1",
        &binder,
    );
    let outer = document.quantifier(
        "filament.op.collection.forall",
        &outer_domain,
        "p1",
        &inner,
    );
    let claim = document.claim(&outer);
    let package = document.package(&[&claim]);
    let translated = translate_analysis_claims(&package).expect("nested quantifiers translate");
    let ExpressionKind::Quantifier {
        local: outer_local,
        predicate,
        ..
    } = expression(&translated, 0).kind()
    else {
        panic!("claim is not quantified")
    };
    let ExpressionKind::Quantifier {
        local: inner_local,
        ..
    } = predicate.kind()
    else {
        panic!("outer predicate is not quantified")
    };
    assert_ne!(outer_local, inner_local);

    let query = lower_analysis_request(translated.request()).expect("nested quantifiers lower");
    assert_eq!(query.quantifiers().len(), 2);
    assert_ne!(query.quantifiers()[0].source, query.quantifiers()[1].source);
}

#[trace("TC-030", "FR-034-AC-2")]
#[test]
fn package_quantifiers_refuse_an_unfixed_collection_at_the_quantifier() {
    let mut document = Document::new();
    let binder = document.parameter();
    let collection_type = document.collection_type("sequence", "scalar_type/boolean", 2);
    let collection = document.parameter_of(&collection_type);
    let quantifier = document.quantifier(
        "filament.op.collection.forall",
        &collection,
        "p1",
        &binder,
    );
    let claim = document.claim(&quantifier);
    let package = document.package(&[&claim]);
    let quantifier_index = index(&package, &quantifier);
    let error = translate_analysis_claims(&package).expect_err("unfixed collection refused");
    assert_eq!(error.kind(), TranslationErrorKind::CannotLower);
    assert_eq!(error.node(), Some(quantifier_index));
}

#[trace("TC-030", "FR-034-AC-8")]
#[test]
fn package_collection_expansion_accepts_1024_and_refuses_1025_elements() {
    for (count, accepted) in [(1_024, true), (1_025, false)] {
        let mut document = Document::new();
        let binder = document.parameter();
        let truth = document.literal(true);
        let values = vec![truth.as_str(); count];
        let collection =
            document.collection_literal("sequence", "scalar_type/boolean", &values);
        let quantifier = document.quantifier(
            "filament.op.collection.forall",
            &collection,
            "p1",
            &binder,
        );
        let claim = document.claim(&quantifier);
        let package = document.package(&[&claim]);
        match translate_analysis_claims(&package) {
            Ok(translation) if accepted => {
                let query = lower_analysis_request(translation.request()).expect("query");
                assert_eq!(query.quantifiers().len(), 1);
                assert_eq!(query.quantifiers()[0].instance_count, 1_024);
            }
            Err(error) if !accepted => {
                assert!(error.message().contains("expansion-ceiling"), "{error}");
                assert!(error.clause().is_some(), "{error}");
            }
            Ok(_) => panic!("{count} elements unexpectedly translated"),
            Err(error) => panic!("{count} elements unexpectedly refused: {error}"),
        }
    }
}

#[trace("TC-030", "FR-034-AC-7")]
#[test]
fn package_quantifiers_refuse_a_text_domain_at_the_quantifier() {
    let mut document = Document::new();
    let text_domain = document.unsupported_type("text");
    let _binder = document.parameter_of(&text_domain);
    let collection = document.collection_literal("sequence", &text_domain, &[]);
    let predicate = document.literal(true);
    let quantifier = document.quantifier(
        "filament.op.collection.forall",
        &collection,
        "p1",
        &predicate,
    );
    let claim = document.claim(&quantifier);
    let package = document.package(&[&claim]);
    let error = translate_analysis_claims(&package).expect_err("text domain refused");
    assert_eq!(error.kind(), TranslationErrorKind::CannotLower);
    assert_eq!(error.node(), Some(index(&package, &quantifier)));
}

#[trace("FR-009-AC-20")]
#[test]
fn claims_become_clauses_of_one_requirement_in_node_order() {
    let mut document = Document::new();
    let p = document.parameter();
    let q = document.parameter();
    let first = document.op(AND, &[&p, &q]);
    let second = document.op(OR, &[&p, &q]);
    let claim_a = document.claim(&first);
    let claim_b = document.claim(&second);
    let (package, translation) = translate(&document, &[&claim_b, &claim_a]);

    let contract = translation.package();
    assert_eq!(contract.requirements().len(), 1);
    let requirement = &contract.requirements()[0];
    assert_eq!(requirement.id().as_str(), "analysis");
    assert_eq!(contract.id().as_str(), PACKAGE);
    let expected = {
        let mut indexes = [index(&package, &claim_a), index(&package, &claim_b)];
        indexes.sort_unstable();
        indexes
    };
    let ids = requirement
        .clauses()
        .iter()
        .map(|clause| clause.id().to_string())
        .collect::<Vec<_>>();
    assert_eq!(
        ids,
        expected.map(|node| format!("claim-{node}")).to_vec(),
        "clauses follow graph order, not request order"
    );
    let reference = contract.requirement_ref(requirement);
    assert_eq!(reference.revision().get(), 1);
    for clause in requirement.clauses() {
        assert!(matches!(
            clause.anchor(),
            Some(ExecutionPoint::Handler { name }) if name.as_str() == "analysis"
        ));
    }
    assert_eq!(translation.environment().owner(), &reference);
    assert_eq!(translation.request().statement_count(), 2);
}

#[trace("FR-009-AC-20")]
#[test]
fn a_package_name_the_analysis_api_refuses_is_named() {
    let mut document = Document::new();
    document.name = "bad name".to_owned();
    let p = document.parameter();
    let claim = document.claim(&p);
    let package = document.package(&[&claim]);
    let error = translate_analysis_claims(&package).expect_err("package name refused");
    assert_eq!(error.kind(), TranslationErrorKind::PackageName);
    assert!(error.message().contains("bad name"), "{error}");
}

#[test]
fn selected_requests_reject_non_claim_and_empty_groups() {
    let mut document = Document::new();
    let parameter = document.parameter();
    let first = document.claim(&parameter);
    let second = document.claim(&parameter);
    let package = document.package(&[&first, &second]);
    let first_index = index(&package, &first);
    let second_index = index(&package, &second);
    let parameter_index = index(&package, &parameter);

    let non_claim = BTreeSet::from([parameter_index]);
    let error = translate_analysis_claims_for_request(
        &package,
        Some(&non_claim),
        RunPackageRequest::Consistency,
    )
    .expect_err("selection names a non-claim node");
    assert_eq!(error.kind(), TranslationErrorKind::Construction);
    assert!(error.message().contains("not an analysis_claim"));

    let empty = BTreeSet::new();
    let error = translate_analysis_claims_for_request(
        &package,
        Some(&empty),
        RunPackageRequest::Consistency,
    )
    .expect_err("consistency needs a selected claim");
    assert!(error
        .message()
        .contains("does not contain an analysis_claim"));

    let all = BTreeSet::from([first_index, second_index]);
    let error = translate_analysis_claims_for_request(
        &package,
        Some(&all),
        RunPackageRequest::Contradiction,
    )
    .expect_err("contradiction needs an approved claim");
    assert!(error.message().contains("one approved claim"));

    let error =
        translate_analysis_claims_for_request(&package, Some(&all), RunPackageRequest::Redundancy)
            .expect_err("redundancy needs one draft and an approved peer");
    assert!(error.message().contains("one draft claim"));
}

#[test]
fn comparison_requests_require_an_explicit_selection() {
    let mut document = Document::new();
    let parameter = document.parameter();
    let claim = document.claim(&parameter);
    let package = document.package(&[&claim]);

    for request in [
        RunPackageRequest::Contradiction,
        RunPackageRequest::Redundancy,
    ] {
        let error = translate_analysis_claims_for_request(&package, None, request)
            .expect_err("comparison needs an explicit selection");
        assert!(error.message().contains("require a selection"));
    }
}

#[test]
fn selected_consistency_translates_a_boolean_literal_claim() {
    let mut document = Document::new();
    let literal = document.literal(true);
    let claim = document.claim(&literal);
    let package = document.package(&[&claim]);
    let selected = BTreeSet::from([index(&package, &claim)]);
    let translation = translate_analysis_claims_for_request(
        &package,
        Some(&selected),
        RunPackageRequest::Consistency,
    )
    .expect("selected Boolean literal");
    assert_eq!(translation.request().statement_count(), 1);
    assert!(matches!(
        expression(&translation, 0).kind(),
        ExpressionKind::BooleanLiteral { value: true }
    ));
    assert_eq!(translation.into_request().statement_count(), 1);
}

#[test]
fn selected_translation_refuses_wrong_operation_arity() {
    let mut document = Document::new();
    let parameter = document.parameter();
    let not = document.op(NOT, &[&parameter]);
    let claim = document.claim(&not);
    let built = document.bytes(&[&claim]);
    let admitted = read_package(&built);
    let operation_node = index(&admitted, &not);
    let claim_node = index(&admitted, &claim);
    let package = mutated(&built, |value| {
        value["semantic_graph"]["nodes"][operation_node]["body"]["arguments"] =
            json!([common::reference(&parameter), common::reference(&parameter)]);
    });
    let selected = BTreeSet::from([claim_node]);
    let error = translate_analysis_claims_for_request(
        &package,
        Some(&selected),
        RunPackageRequest::Consistency,
    )
    .expect_err("NOT only accepts one operand");
    assert_eq!(error.kind(), TranslationErrorKind::CannotLower);
    assert!(error.message().contains("takes 1 arguments"));
}

#[trace("FR-009-AC-21")]
#[test]
fn each_boolean_operator_translates_to_its_expression_kind() {
    let binary = [
        (AND, Some(BooleanOperator::TotalAnd), None),
        (OR, Some(BooleanOperator::TotalOr), None),
        (IMPLIES, Some(BooleanOperator::Implication), None),
        (EQ, None, Some(ComparisonOperator::Equal)),
        (NE, None, Some(ComparisonOperator::NotEqual)),
    ];
    for (identity, boolean, compare) in binary {
        let mut document = Document::new();
        let p = document.parameter();
        let q = document.parameter();
        let application = document.op(identity, &[&p, &q]);
        let claim = document.claim(&application);
        let (package, translation) = translate(&document, &[&claim]);
        let (left, right) = (value_reference(&package, &p), value_reference(&package, &q));
        match expression(&translation, 0).kind() {
            ExpressionKind::Boolean {
                operator,
                left: l,
                right: r,
            } => {
                assert_eq!(Some(*operator), boolean, "{identity}");
                assert_eq!((l.kind(), r.kind()), (&left, &right), "{identity}");
            }
            ExpressionKind::Compare {
                operator,
                left: l,
                right: r,
            } => {
                assert_eq!(Some(*operator), compare, "{identity}");
                assert_eq!((l.kind(), r.kind()), (&left, &right), "{identity}");
            }
            other => panic!("{identity} translated to {other:?}"),
        }
    }

    let mut document = Document::new();
    let p = document.parameter();
    let not = document.op(NOT, &[&p]);
    let claim = document.claim(&not);
    let (package, translation) = translate(&document, &[&claim]);
    match expression(&translation, 0).kind() {
        ExpressionKind::BooleanNot { operand } => {
            assert_eq!(operand.kind(), &value_reference(&package, &p));
        }
        other => panic!("not translated to {other:?}"),
    }
}

#[trace("FR-009-AC-21")]
#[test]
fn a_boolean_literal_and_a_reference_chain_translate_to_the_target_body() {
    let mut document = Document::new();
    let truth = document.literal(true);
    let claim = document.claim(&truth);
    let (_, translation) = translate(&document, &[&claim]);
    assert_eq!(
        expression(&translation, 0).kind(),
        &ExpressionKind::BooleanLiteral { value: true }
    );

    let mut document = Document::new();
    let p = document.parameter();
    let not = document.op(NOT, &[&p]);
    let hop = document.reference_node(&not, "scalar_type/boolean");
    let hop_again = document.reference_node(&hop, "scalar_type/boolean");
    let claim = document.claim(&hop_again);
    let (package, translation) = translate(&document, &[&claim]);
    match expression(&translation, 0).kind() {
        ExpressionKind::BooleanNot { operand } => {
            assert_eq!(operand.kind(), &value_reference(&package, &p));
        }
        other => panic!("reference chain translated to {other:?}"),
    }
}

#[trace("FR-009-AC-22")]
#[test]
fn a_parameter_reached_twice_is_one_declaration_and_one_symbol() {
    let mut document = Document::new();
    let p = document.parameter();
    let q = document.parameter();
    let first = document.op(NOT, &[&p]);
    let second = document.op(AND, &[&p, &q]);
    let claim_a = document.claim(&first);
    let claim_b = document.claim(&second);
    let _unused = document.parameter();
    let (package, translation) = translate(&document, &[&claim_a, &claim_b]);

    let declarations = translation.environment().values();
    let mut expected = [index(&package, &p), index(&package, &q)];
    expected.sort_unstable();
    let names = declarations
        .iter()
        .map(|declaration| declaration.name().as_str().to_owned())
        .collect::<Vec<_>>();
    assert_eq!(names, expected.map(|node| format!("n{node}")).to_vec());
    for declaration in declarations {
        assert_eq!(declaration.kind(), ValueDeclarationKind::Input);
        assert_eq!(declaration.value_type(), &ValueType::Boolean);
    }
    let query = lower_analysis_request(translation.request()).expect("lowers");
    assert_eq!(
        query.variables().len(),
        2,
        "one symbol per distinct parameter"
    );
}

#[trace("FR-009-AC-24")]
#[test]
fn spans_come_from_the_first_region_of_the_first_occurrence() {
    let mut document = Document::new();
    let p = document.parameter();
    let not = document.op(NOT, &[&p]);
    let claim = document.claim(&not);
    let (package, translation) = translate(&document, &[&claim]);

    let check = |span: &filament_contract_model::SourceSpan, label: &str| {
        let (start, end) = document.region(label);
        assert_eq!((span.start().line(), span.start().column()), (1, 1));
        assert_eq!((span.end().line(), span.end().column()), (1, 1));
        assert_eq!(span.start().byte_offset(), start);
        assert_eq!(span.end().byte_offset(), end);
        assert_eq!(span.source().document().as_str(), "contract-package");
        assert_eq!(span.source().revision().get(), 1);
    };
    let requirement = &translation.package().requirements()[0];
    let clause = &requirement.clauses()[0];
    check(clause.source(), &claim);
    let root = expression(&translation, 0);
    check(root.source(), &claim);
    match root.kind() {
        ExpressionKind::BooleanNot { operand } => check(operand.source(), &claim),
        other => panic!("{other:?}"),
    }
    check(translation.environment().values()[0].source(), &p);
    assert_eq!(
        translation.environment().values()[0].name().as_str(),
        symbol(&package, &p)
    );
    let start = requirement.source().start();
    assert_eq!(
        (
            start.byte_offset(),
            requirement.source().end().byte_offset()
        ),
        (0, 0)
    );
}

#[trace("FR-009-AC-26")]
#[test]
fn two_translations_of_the_same_bytes_agree() {
    let mut document = Document::new();
    let p = document.parameter();
    let q = document.parameter();
    let and = document.op(AND, &[&p, &q]);
    let claim_a = document.claim(&and);
    let claim_b = document.claim(&p);
    let bytes = document.bytes(&[&claim_a, &claim_b]);
    let read = || match ContractPackageWireV1::read(
        &bytes,
        CheckedPackageReadLimits::bounded(),
        &Default::default(),
    ) {
        ContractPackageReadResult::Admitted(package) => *package,
        other => panic!("{other:?}"),
    };
    let first = translate_analysis_claims(&read()).expect("first");
    let second = translate_analysis_claims(&read()).expect("second");
    assert_eq!(first.package(), second.package());
    assert_eq!(first.environment(), second.environment());
}

#[trace("FR-009-AC-23")]
#[test]
fn a_package_without_a_claim_or_with_a_non_boolean_claim_is_distinguished() {
    let mut document = Document::new();
    let p = document.parameter();
    let package = document.package(&[&p]);
    let error = translate_analysis_claims(&package).expect_err("no claim");
    assert_eq!(error.kind(), TranslationErrorKind::NoAnalysableClause);

    let mut document = Document::new();
    let range = document.range();
    let number = document.parameter_of(&range);
    let claim = document.claim(&number);
    let (package, error) = refusal(&document, &[&claim]);
    assert_eq!(error.kind(), TranslationErrorKind::NonBooleanClaim);
    assert_eq!(error.node(), Some(index(&package, &claim)));
}

fn assert_refused_at(
    package: &ContractPackageV1,
    error: &TranslationError,
    kind: TranslationErrorKind,
    label: &str,
) {
    let node = index(package, label);
    assert_eq!(error.kind(), kind, "{error}");
    assert_eq!(error.node(), Some(node), "{error}");
    assert_eq!(error.path(), format!("semantic_graph.nodes[{node}]"));
    assert!(error
        .to_string()
        .contains(&format!("semantic_graph.nodes[{node}]")));
}

#[trace("FR-009-AC-23")]
#[test]
fn an_operation_outside_the_supported_subset_is_refused_at_its_node() {
    let mut document = Document::new();
    let range = document.range();
    let a = document.parameter_of(&range);
    let b = document.parameter_of(&range);
    let other = document.application("filament.op.integer.add", &[&a, &b], &range);
    let less = document.application(
        "filament.op.integer.lt",
        &[&other, &b],
        "scalar_type/boolean",
    );
    let claim = document.claim(&less);
    let bytes = with_identity(
        &document.bytes(&[&claim]),
        &other,
        "filament.op.integer.min",
    );
    let package = read_package(&bytes);
    let error = translate_analysis_claims(&package).expect_err("refused");
    assert_refused_at(&package, &error, TranslationErrorKind::CannotLower, &other);
}

#[trace("FR-009-AC-23")]
#[test]
fn the_depth_bound_is_128_against_129() {
    let mut document = Document::new();
    let (claim, _) = not_chain(&mut document, 127);
    translate(&document, &[&claim]);

    let mut document = Document::new();
    let (claim, innermost) = not_chain(&mut document, 128);
    let (package, error) = refusal(&document, &[&claim]);
    assert_refused_at(
        &package,
        &error,
        TranslationErrorKind::ExpressionDepthBound,
        &innermost,
    );
}

#[trace("FR-009-AC-23")]
#[test]
fn the_node_bound_is_10000_against_10001() {
    let mut document = Document::new();
    let mut sized = Sized::new(&mut document);
    let body = sized.node(&mut document, 9_998);
    let top = document.op(AND, &[&body, &sized.parameter]);
    let claim = document.claim(&top);
    translate(&document, &[&claim]);

    let mut document = Document::new();
    let mut sized = Sized::new(&mut document);
    let body = sized.node(&mut document, 9_998);
    let last = document.op(NOT, &[&sized.parameter]);
    let top = document.op(AND, &[&body, &last]);
    let claim = document.claim(&top);
    let (package, error) = refusal(&document, &[&claim]);
    assert_refused_at(
        &package,
        &error,
        TranslationErrorKind::ExpressionNodeBound,
        &last,
    );
}

#[trace("FR-009-AC-23")]
#[test]
fn a_shared_subterm_chain_of_61_nodes_is_refused_without_expanding_it() {
    let mut document = Document::new();
    let p = document.parameter();
    let mut top = document.op(AND, &[&p, &p]);
    for _ in 1..61 {
        top = document.op(AND, &[&top, &top]);
    }
    let claim = document.claim(&top);
    let package = document.package(&[&claim]);
    let started = std::time::Instant::now();
    let error = translate_analysis_claims(&package).expect_err("2^61 expansion refused");
    assert_eq!(
        error.kind(),
        TranslationErrorKind::ExpressionNodeBound,
        "{error}"
    );
    assert!(error.node().is_some());
    assert!(
        started.elapsed() < std::time::Duration::from_secs(5),
        "{:?}",
        started.elapsed()
    );
}

/// The admitted package read from the canonical bytes of `built` after `edit` changed them.
fn mutated(built: &[u8], edit: impl FnOnce(&mut Value)) -> ContractPackageV1 {
    let mut value: Value = serde_json::from_slice(built).expect("package json");
    edit(&mut value);
    let bytes = serde_json::to_vec(&value).expect("package bytes");
    match ContractPackageWireV1::read(
        &bytes,
        CheckedPackageReadLimits::bounded(),
        &Default::default(),
    ) {
        ContractPackageReadResult::Admitted(package) => *package,
        other => panic!("mutated package refused: {other:?}"),
    }
}

#[trace("FR-009-AC-24")]
#[test]
fn a_claim_node_without_occurrences_is_refused_at_the_node() {
    let mut document = Document::new();
    let p = document.parameter();
    let claim = document.claim(&p);
    let built = document.package(&[&claim]);
    let node = index(&built, &claim);
    let package = mutated(&document.bytes(&[&claim]), |value| {
        value["semantic_graph"]["nodes"][node]["occurrences"] = json!([]);
        value["source_map"]["nodes"][node]["occurrences"] = json!([]);
        value["source_map"]["requested_items"][0]["occurrences"] = json!([]);
    });
    let error = translate_analysis_claims(&package).expect_err("no occurrence");
    assert_refused_at(&package, &error, TranslationErrorKind::SourceRegion, &claim);
}

#[trace("FR-009-AC-23")]
#[test]
fn an_integer_typed_parameter_operand_is_refused_at_the_parameter() {
    let mut document = Document::new();
    let p = document.parameter();
    let q = document.parameter();
    let and = document.op(AND, &[&p, &q]);
    let claim = document.claim(&and);
    let built = document.package(&[&claim]);
    let node = index(&built, &p);
    let package = mutated(&document.bytes(&[&claim]), |value| {
        value["semantic_graph"]["nodes"][node]["semantic_type"] = id("scalar_type/integer");
    });
    let error = translate_analysis_claims(&package).expect_err("integer parameter");
    assert_refused_at(&package, &error, TranslationErrorKind::CannotLower, &p);
    assert!(error.message().contains("not Boolean"), "{error}");
}

#[trace("FR-009-AC-23")]
#[test]
fn claims_sharing_one_large_tree_are_bounded_in_total() {
    let mut document = Document::new();
    let mut sized = Sized::new(&mut document);
    let top = sized.node(&mut document, 10_000);
    let claims = (0..11).map(|_| document.claim(&top)).collect::<Vec<_>>();
    let labels = claims.iter().map(String::as_str).collect::<Vec<_>>();
    let (package, error) = refusal(&document, &labels);
    assert_refused_at(
        &package,
        &error,
        TranslationErrorKind::ExpressionTotalBound,
        &top,
    );
}

#[trace("FR-009-AC-23")]
#[test]
fn a_reference_to_another_claim_is_refused_at_the_claim() {
    let mut document = Document::new();
    let p = document.parameter();
    let inner = document.claim(&p);
    let outer = document.claim(&inner);
    let (package, error) = refusal(&document, &[&outer]);
    assert_refused_at(&package, &error, TranslationErrorKind::CannotLower, &inner);
}

#[trace("FR-009-AC-23")]
#[test]
fn a_reference_cycle_is_refused() {
    let (document, claim, [first, second]) = cycle_document();
    let (package, error) = refusal(&document, &[&claim]);
    assert_eq!(error.kind(), TranslationErrorKind::CannotLower, "{error}");
    assert!(error.message().contains("cycle"), "{error}");
    let named = error.node().expect("a node");
    assert!([index(&package, &first), index(&package, &second)].contains(&named));
}
