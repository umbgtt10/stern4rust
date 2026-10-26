// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// One dependency as a member manifest declares it.
//
// Plain data, read from the TOML rather than from `cargo metadata`, because the
// question is *how it was written* and not what it resolves to. `cargo metadata`
// answers the second and erases the first: a dependency taken from the workspace
// and one spelled out in the member look identical once resolved.
//
// Beside its four facts -- manifest, name, section, whether it takes from the
// workspace -- it carries what it asks of the crate it names: a target, a
// version requirement, features, default features, optional. Those are what
// tell a dev-dependency that adds something from one that only repeats.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManifestDependency {
    manifest: String,
    name: String,
    section: String,
    takes_from_workspace: bool,
    target: Option<String>,
    version: Option<String>,
    features: Vec<String>,
    default_features: bool,
    optional: bool,
}

impl ManifestDependency {
    pub const DEPENDENCIES: &'static str = "dependencies";
    pub const DEV_DEPENDENCIES: &'static str = "dev-dependencies";
    pub const SECTIONS: [&'static str; 3] =
        ["dependencies", "dev-dependencies", "build-dependencies"];

    // The subset declared in one manifest.
    //
    // `check_workspace` runs once per package, so handing every package the
    // whole workspace's declarations stated each finding once per member --
    // twenty real findings became 580 in a twenty-nine member workspace, every
    // copy identical, the count simply tracking the member count. A finding
    // about `alpha/Cargo.toml` belongs to `alpha`, and the loop walking the
    // packages already knows which one it is in.
    //
    // `None` stays `None`: that is how a package outside a workspace tells the
    // rule to stand down, and filtering must not turn it into an empty list.
    pub fn in_manifest(all: Option<&[Self]>, manifest: &str) -> Option<Vec<Self>> {
        Some(
            all?.iter()
                .filter(|dependency| dependency.manifest == manifest)
                .cloned()
                .collect(),
        )
    }

    // Asking for the crate and nothing more: default features on, not optional,
    // no features, no target, no version requirement.
    pub fn new(manifest: &str, name: &str, section: &str, takes_from_workspace: bool) -> Self {
        Self {
            manifest: manifest.to_string(),
            name: name.to_string(),
            section: section.to_string(),
            takes_from_workspace,
            target: None,
            version: None,
            features: Vec::new(),
            default_features: true,
            optional: false,
        }
    }

    pub fn default_features(&self) -> bool {
        self.default_features
    }

    pub fn features(&self) -> &[String] {
        &self.features
    }

    pub fn manifest(&self) -> &str {
        &self.manifest
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn optional(&self) -> bool {
        self.optional
    }

    // Whether this entry, a dev-dependency, repeats `normal`, a dependency, and
    // adds nothing to it -- so the tests would have exactly what they have
    // without it.
    //
    // Cargo forbids naming one package under two keys, and one key for two
    // sources, so a repeat always shares the name; a renamed dependency is
    // matched on it like any other. The dependency must cover the dev entry's
    // target: untargeted, it covers all of them, and a target is compared as
    // written. The two must be written alike -- both taken from the workspace or
    // neither, one version requirement -- or what one of them enables is not in
    // this manifest to compare.
    pub fn repeats(&self, normal: &Self) -> bool {
        self.pairs_with(normal) && self.written_like(normal) && !self.adds_to(normal)
    }

    pub fn section(&self) -> &str {
        &self.section
    }

    // The table the entry sits in, as a manifest spells its header.
    pub fn table(&self) -> String {
        self.target.as_ref().map_or_else(
            || self.section.clone(),
            |target| format!("target.'{target}'.{}", self.section),
        )
    }

    pub fn takes_from_workspace(&self) -> bool {
        self.takes_from_workspace
    }

    pub fn target(&self) -> Option<&str> {
        self.target.as_deref()
    }

    pub fn version(&self) -> Option<&str> {
        self.version.as_deref()
    }

    pub fn with_default_features(self, default_features: bool) -> Self {
        Self {
            default_features,
            ..self
        }
    }

    pub fn with_features(self, features: Vec<String>) -> Self {
        Self { features, ..self }
    }

    pub fn with_optional(self, optional: bool) -> Self {
        Self { optional, ..self }
    }

    pub fn with_target(self, target: Option<&str>) -> Self {
        Self {
            target: target.map(str::to_string),
            ..self
        }
    }

    pub fn with_version(self, version: Option<&str>) -> Self {
        Self {
            version: version.map(str::to_string),
            ..self
        }
    }

    // What the tests would lose without this entry: the crate itself where the
    // dependency is optional, the default features where it turns them off, a
    // feature it does not list.
    fn adds_to(&self, normal: &Self) -> bool {
        normal.optional
            || (self.default_features && !normal.default_features)
            || self
                .features
                .iter()
                .any(|feature| !normal.features.contains(feature))
    }

    // A dev-dependency and a dependency of one manifest, naming one crate, the
    // dependency covering the dev entry's target.
    fn pairs_with(&self, normal: &Self) -> bool {
        self.section == Self::DEV_DEPENDENCIES
            && normal.section == Self::DEPENDENCIES
            && self.manifest == normal.manifest
            && self.name == normal.name
            && (normal.target.is_none() || normal.target == self.target)
    }

    fn written_like(&self, normal: &Self) -> bool {
        self.takes_from_workspace == normal.takes_from_workspace && self.version == normal.version
    }
}
