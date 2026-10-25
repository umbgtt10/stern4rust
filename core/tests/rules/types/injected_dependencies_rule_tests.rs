// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// A type does not build its own dependencies: a field declared as a trait
// receives only what the caller passed in. Judged in every constructor, every
// Default, From and TryFrom impl, every value a constructor hands another
// constructor of its type, and every `self.field = ...` elsewhere. A builder --
// a type whose `build(self)` returns another type -- composes, and is not judged.

use stern4rust::reporting::offence::Offence;
use stern4rust::rule::Rule;
use stern4rust::rules::types::injected_dependencies_rule::InjectedDependenciesRule;
use stern4rust::source_file::SourceFile;

const APP: &str = "pub struct App {\n    reporter: Box<dyn Reporter>,\n    size: usize,\n}\n\n";
const RULE: &str = "injected-dependencies";

fn app(implementation: &str) -> String {
    format!("{APP}{implementation}")
}

fn check(files: &[(&str, &str)]) -> Vec<Offence> {
    let sources: Vec<SourceFile> = files
        .iter()
        .map(|(path, contents)| SourceFile::new(path, contents))
        .collect();
    InjectedDependenciesRule::new().check_workspace(&sources)
}

#[test]
fn check_of_a_single_file_reports_nothing() {
    // Arrange
    let file = SourceFile::new(
        "src/app.rs",
        &app(
            "impl App {\n    pub fn new() -> Self {\n        Self { reporter: Box::new(StdoutReporter::new()), size: 0 }\n    }\n}\n",
        ),
    );

    // Act
    let offences = InjectedDependenciesRule::new().check(&file);

    // Assert
    assert!(offences.is_empty(), "expected none, got {offences:?}");
}

#[test]
fn check_workspace_of_a_builder_building_its_parts_reports_nothing() {
    // Arrange & Act
    let offences = check(&[(
        "src/app_builder.rs",
        "pub struct AppBuilder {\n    reporter: Option<Box<dyn Reporter>>,\n}\n\nimpl AppBuilder {\n    pub fn new() -> Self {\n        Self { reporter: Some(Box::new(StdoutReporter::new())) }\n    }\n\n    pub fn quiet(self) -> Self {\n        Self { reporter: Some(Box::new(NullReporter)) }\n    }\n\n    pub fn build(self) -> App {\n        App::new(self.reporter.unwrap())\n    }\n}\n",
    )]);

    // Assert
    assert!(offences.is_empty(), "expected none, got {offences:?}");
}

#[test]
fn check_workspace_of_a_concrete_field_built_inside_reports_nothing() {
    // Arrange & Act
    let offences = check(&[(
        "src/app.rs",
        "pub struct App {\n    store: Store,\n}\n\nimpl App {\n    pub fn new() -> Self {\n        Self { store: Store::new() }\n    }\n}\n",
    )]);

    // Assert
    assert!(offences.is_empty(), "expected none, got {offences:?}");
}

#[test]
fn check_workspace_of_a_constructor_building_a_field_in_two_arms_reports_it_once_at_the_first() {
    // Arrange & Act
    let offences = check(&[(
        "src/app.rs",
        &app(
            "impl App {\n    pub fn new(quiet: bool) -> Self {\n        if quiet {\n            Self { reporter: Box::new(NullReporter), size: 0 }\n        } else {\n            Self { reporter: Box::new(StdoutReporter::new()), size: 0 }\n        }\n    }\n}\n",
        ),
    )]);

    // Assert
    assert_eq!(offences.len(), 1);
    assert_eq!(offences[0].line(), 9);
}

#[test]
fn check_workspace_of_a_constructor_building_a_trait_field_says_what_to_do() {
    // Arrange & Act
    let offences = check(&[(
        "src/app.rs",
        &app(
            "impl App {\n    pub fn new() -> Self {\n        Self {\n            reporter: Box::new(StdoutReporter::new()),\n            size: 0,\n        }\n    }\n}\n",
        ),
    )]);

    // Assert
    assert_eq!(offences.len(), 1);
    assert_eq!(offences[0].rule(), RULE);
    assert_eq!(offences[0].file(), "src/app.rs");
    assert_eq!(offences[0].line(), 9);
    assert_eq!(offences[0].subject(), Some("App::new"));
    assert_eq!(
        offences[0].description(),
        "`App::new` builds `StdoutReporter` for `reporter`, a field declared as a trait"
    );
    assert_eq!(
        offences[0].correction(),
        "take it as a parameter and build it where the parts are composed: main, a builder, a test"
    );
}

#[test]
fn check_workspace_of_a_constructor_destructuring_its_parts_from_a_parameter_reports_nothing() {
    // Arrange & Act
    let offences = check(&[(
        "src/app.rs",
        &app(
            "impl App {\n    pub fn new(parts: AppParts) -> Self {\n        let AppParts { reporter, size } = parts;\n        Self { reporter, size }\n    }\n}\n",
        ),
    )]);

    // Assert
    assert!(offences.is_empty(), "expected none, got {offences:?}");
}

#[test]
fn check_workspace_of_a_constructor_filling_a_trait_field_from_a_built_local_names_what_it_built() {
    // Arrange & Act
    let offences = check(&[(
        "src/app.rs",
        &app(
            "impl App {\n    pub fn new() -> Self {\n        let reporter = Box::new(StdoutReporter::new());\n        Self { reporter, size: 0 }\n    }\n}\n",
        ),
    )]);

    // Assert
    assert_eq!(offences.len(), 1);
    assert_eq!(
        offences[0].description(),
        "`App::new` builds `StdoutReporter` for `reporter`, a field declared as a trait"
    );
}

#[test]
fn check_workspace_of_a_constructor_handing_a_built_part_to_another_constructor_says_what_to_do() {
    // Arrange & Act
    let offences = check(&[(
        "src/app.rs",
        &app(
            "impl App {\n    pub fn new() -> Self {\n        Self::with_deps(Box::new(StdoutReporter::new()), 0)\n    }\n\n    pub fn with_deps(reporter: Box<dyn Reporter>, size: usize) -> Self {\n        Self { reporter, size }\n    }\n}\n",
        ),
    )]);

    // Assert
    assert_eq!(offences.len(), 1);
    assert_eq!(offences[0].line(), 8);
    assert_eq!(
        offences[0].description(),
        "`App::new` passes `StdoutReporter`, which it built, to `App::with_deps`"
    );
    assert_eq!(
        offences[0].correction(),
        "make with_deps the constructor and compose the parts in main"
    );
}

#[test]
fn check_workspace_of_a_constructor_handing_built_parts_over_twice_reports_each_parameter_once() {
    // Arrange & Act
    let offences = check(&[(
        "src/app.rs",
        &app(
            "impl App {\n    pub fn new(quiet: bool) -> Self {\n        if quiet {\n            Self::with_deps(Box::new(NullReporter), 0)\n        } else {\n            Self::with_deps(Box::new(StdoutReporter::new()), 1)\n        }\n    }\n\n    pub fn with_deps(reporter: Box<dyn Reporter>, size: usize) -> Self {\n        Self { reporter, size }\n    }\n}\n",
        ),
    )]);

    // Assert
    assert_eq!(offences.len(), 1);
}

#[test]
fn check_workspace_of_a_constructor_repackaging_its_parameters_reports_nothing() {
    // Arrange & Act
    let offences = check(&[(
        "src/app.rs",
        &app(
            "impl App {\n    pub fn new(reporter: Option<Box<dyn Reporter>>, deps: Deps) -> Result<Self, Error> {\n        let parts = Self::required(reporter)?;\n        let parts = parts.into_parts();\n        let AppParts { size, .. } = deps.into_parts(3);\n        Ok(Self { reporter: parts.reporter, size })\n    }\n}\n",
        ),
    )]);

    // Assert
    assert!(offences.is_empty(), "expected none, got {offences:?}");
}

#[test]
fn check_workspace_of_a_constructor_taking_every_part_in_each_passed_in_shape_reports_nothing() {
    // Arrange & Act
    let offences = check(&[(
        "src/app.rs",
        "pub struct App {\n    a: Box<dyn A>,\n    b: Option<Arc<dyn B>>,\n    c: Rc<dyn C>,\n    d: Option<Box<dyn D>>,\n    e: Box<dyn E>,\n}\n\nimpl App {\n    pub fn new(a: Box<dyn A>, b: &Arc<dyn B>, c: &Rc<dyn C>, e: impl E) -> Self {\n        Self { a, b: Some(Arc::clone(b)), c: c.clone(), d: None, e: Box::new(e) }\n    }\n}\n",
    )]);

    // Assert
    assert!(offences.is_empty(), "expected none, got {offences:?}");
}

#[test]
fn check_workspace_of_a_default_impl_building_a_trait_field_reports_it() {
    // Arrange & Act
    let offences = check(&[(
        "src/app.rs",
        &app(
            "impl Default for App {\n    fn default() -> Self {\n        Self { reporter: Box::new(StdoutReporter::new()), size: 0 }\n    }\n}\n",
        ),
    )]);

    // Assert
    assert_eq!(offences.len(), 1);
    assert_eq!(offences[0].subject(), Some("App::default"));
}

#[test]
fn check_workspace_of_a_file_that_does_not_parse_reports_nothing() {
    // Arrange & Act
    let offences = check(&[(
        "src/app.rs",
        "pub struct App {\n    reporter: Box<dyn Reporter>,\n",
    )]);

    // Assert
    assert!(offences.is_empty(), "expected none, got {offences:?}");
}

#[test]
fn check_workspace_of_a_from_impl_reading_a_trait_field_from_its_source_reports_it() {
    // Arrange & Act
    let offences = check(&[(
        "src/app.rs",
        &app(
            "impl From<Settings> for App {\n    fn from(settings: Settings) -> Self {\n        Self { reporter: settings.reporter(), size: 0 }\n    }\n}\n",
        ),
    )]);

    // Assert
    assert_eq!(offences.len(), 1);
    assert_eq!(
        offences[0].description(),
        "`App::from` builds `settings.reporter()` for `reporter`, a field declared as a trait"
    );
}

#[test]
fn check_workspace_of_a_generic_field_bounded_in_a_where_clause_built_inside_reports_it() {
    // Arrange & Act
    let offences = check(&[(
        "src/node.rs",
        "pub struct Node<S>\nwhere\n    S: Storage,\n{\n    store: S,\n}\n\nimpl<S: Storage> Node<S> {\n    pub fn new() -> Self {\n        Self { store: S::open() }\n    }\n}\n",
    )]);

    // Assert
    assert_eq!(offences.len(), 1);
    assert_eq!(
        offences[0].description(),
        "`Node::new` builds `S::open()` for `store`, a field declared as a trait"
    );
}

#[test]
fn check_workspace_of_a_generic_field_bounded_inline_built_inside_reports_it() {
    // Arrange & Act
    let offences = check(&[(
        "src/node.rs",
        "pub struct Node<S: Storage> {\n    store: S,\n}\n\nimpl<S: Storage> Node<S> {\n    pub fn new() -> Self {\n        Self { store: S::default() }\n    }\n}\n",
    )]);

    // Assert
    assert_eq!(offences.len(), 1);
}

#[test]
fn check_workspace_of_a_method_assigning_a_built_value_to_a_trait_field_says_what_to_do() {
    // Arrange & Act
    let offences = check(&[(
        "src/app.rs",
        &app(
            "impl App {\n    pub fn reset(&mut self) {\n        self.reporter = Box::new(NullReporter);\n    }\n}\n",
        ),
    )]);

    // Assert
    assert_eq!(offences.len(), 1);
    assert_eq!(offences[0].line(), 8);
    assert_eq!(offences[0].subject(), Some("App::reset"));
    assert_eq!(
        offences[0].description(),
        "`App::reset` assigns `NullReporter` to `reporter`, a field declared as a trait"
    );
    assert_eq!(
        offences[0].correction(),
        "take the replacement as a parameter"
    );
}

#[test]
fn check_workspace_of_a_method_assigning_a_parameter_to_a_trait_field_reports_nothing() {
    // Arrange & Act
    let offences = check(&[(
        "src/app.rs",
        &app(
            "impl App {\n    pub fn replace(&mut self, reporter: Box<dyn Reporter>) {\n        self.reporter = reporter;\n    }\n}\n",
        ),
    )]);

    // Assert
    assert!(offences.is_empty(), "expected none, got {offences:?}");
}

#[test]
fn check_workspace_of_a_method_assigning_a_trait_field_twice_reports_it_once() {
    // Arrange & Act
    let offences = check(&[(
        "src/app.rs",
        &app(
            "impl App {\n    pub fn reset(&mut self) {\n        self.reporter = Box::new(NullReporter);\n        self.reporter = Box::new(StdoutReporter::new());\n    }\n}\n",
        ),
    )]);

    // Assert
    assert_eq!(offences.len(), 1);
}

#[test]
fn check_workspace_of_a_struct_under_tests_reports_nothing() {
    // Arrange & Act
    let offences = check(&[(
        "tests/support.rs",
        &app(
            "impl App {\n    pub fn new() -> Self {\n        Self { reporter: Box::new(StdoutReporter::new()), size: 0 }\n    }\n}\n",
        ),
    )]);

    // Assert
    assert!(offences.is_empty(), "expected none, got {offences:?}");
}

#[test]
fn check_workspace_of_a_tuple_struct_building_its_trait_field_reports_it_by_position() {
    // Arrange & Act
    let offences = check(&[(
        "src/port.rs",
        "pub struct Port(Box<dyn Link>);\n\nimpl Port {\n    pub fn open() -> Self {\n        Self(Box::new(TcpLink::new()))\n    }\n}\n",
    )]);

    // Assert
    assert_eq!(offences.len(), 1);
    assert_eq!(
        offences[0].description(),
        "`Port::open` builds `TcpLink` for `0`, a field declared as a trait"
    );
}

#[test]
fn check_workspace_of_an_impl_in_another_file_reports_it_there() {
    // Arrange & Act
    let offences = check(&[
        ("src/app.rs", APP),
        (
            "src/wiring.rs",
            "impl App {\n    pub fn new() -> Self {\n        Self { reporter: Box::new(StdoutReporter::new()), size: 0 }\n    }\n}\n",
        ),
    ]);

    // Assert
    assert_eq!(offences.len(), 1);
    assert_eq!(offences[0].file(), "src/wiring.rs");
}

#[test]
fn explanation_shows_a_constructor_building_its_part_and_the_one_taking_it() {
    // Arrange & Act
    let explanation = InjectedDependenciesRule::new().explanation();

    // Assert
    assert_eq!(explanation.name, RULE);
    assert!(
        explanation
            .breaks
            .contains("Box::new(StdoutReporter::new())")
    );
    assert!(explanation.instead.contains("reporter: Box<dyn Reporter>"));
}

#[test]
fn is_configured_for_a_rule_that_needs_nothing_returns_true() {
    // Arrange & Act
    let configured = InjectedDependenciesRule::new().is_configured();

    // Assert
    assert!(configured);
}

#[test]
fn name_is_the_kebab_case_rule_name_used_in_the_report() {
    // Arrange & Act
    let name = InjectedDependenciesRule::new().name();

    // Assert
    assert_eq!(name, RULE);
}

#[test]
fn requirement_of_a_rule_that_needs_nothing_is_none() {
    // Arrange & Act
    let requirement = InjectedDependenciesRule::new().requirement();

    // Assert
    assert!(requirement.is_none());
}
