// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;

use crate::finding::parsing::shapes::method_shape::MethodShape;
use crate::finding::parsing::shapes::type_shape::TypeShape;
use crate::finding::parsing::shapes::value_traits::ValueTraits;

#[derive(Debug, Clone)]
pub struct ShapedType {
    name: String,
    path: String,
    line: usize,
    getters: Vec<String>,
    constructors: usize,
    steps: usize,
    builds: usize,
    behaviours: usize,
}

impl ShapedType {
    pub fn new(name: &str, path: &str, line: usize) -> Self {
        Self {
            name: name.to_string(),
            path: path.to_string(),
            line,
            getters: Vec::new(),
            constructors: 0,
            steps: 0,
            builds: 0,
            behaviours: 0,
        }
    }

    pub fn accessors(&self) -> usize {
        self.getters.len() + self.constructors + self.steps
    }

    pub fn constructors(&self) -> usize {
        self.constructors
    }

    pub fn getters(&self) -> &[String] {
        &self.getters
    }

    pub fn line(&self) -> usize {
        self.line
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn path(&self) -> &str {
        &self.path
    }

    pub fn shape(&self) -> TypeShape {
        if self.builds > 0 {
            TypeShape::Builder
        } else if self.behaviours > 0 {
            TypeShape::Object
        } else {
            TypeShape::Record
        }
    }

    pub fn steps(&self) -> usize {
        self.steps
    }

    pub fn with_method(self, name: &str, shape: MethodShape) -> Self {
        match shape {
            MethodShape::Getter => {
                let mut getters = self.getters;
                getters.push(name.to_string());
                Self { getters, ..self }
            }
            MethodShape::PlainConstructor => Self {
                constructors: self.constructors + 1,
                ..self
            },
            MethodShape::Step => Self {
                steps: self.steps + 1,
                ..self
            },
            MethodShape::Build => Self {
                builds: self.builds + 1,
                ..self
            },
            MethodShape::Behaviour => Self {
                behaviours: self.behaviours + 1,
                ..self
            },
        }
    }

    pub fn with_trait(self, trait_name: &str) -> Self {
        if ValueTraits::contains(trait_name) {
            self
        } else {
            Self {
                behaviours: self.behaviours + 1,
                ..self
            }
        }
    }
}
