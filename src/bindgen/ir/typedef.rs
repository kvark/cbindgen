/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::collections::HashMap;
use std::io::Write;

use syn;

use bindgen::config::Config;
use bindgen::dependencies::Dependencies;
use bindgen::ir::{AnnotationSet, Documentation, Path, Type};
use bindgen::library::Library;
use bindgen::monomorph::Monomorphs;
use bindgen::writer::{Source, SourceWriter};

/// A type alias that is represented as a C typedef
#[derive(Debug, Clone)]
pub struct Typedef {
    pub name: String,
    pub annotations: AnnotationSet,
    pub aliased: Type,
    pub documentation: Documentation,
}

impl Typedef {
    pub fn load(name: String,
                annotations: AnnotationSet,
                ty: &syn::Ty,
                doc: String) -> Result<Typedef, String> {
        if let Some(x) = Type::load(ty)? {
            Ok(Typedef {
                name: name,
                annotations: annotations,
                aliased: x,
                documentation: Documentation::load(doc),
            })
        } else {
            Err(format!("cannot have a typedef of a zero sized type"))
        }
    }

    pub fn transfer_annotations(&mut self, out: &mut HashMap<Path, AnnotationSet>) {
        if self.annotations.is_empty() {
            return;
        }

        match self.aliased.get_root_path() {
            Some(alias_path) => {
                if out.contains_key(&alias_path) {
                    warn!("multiple typedef's with annotations for {}. ignoring annotations from {}.",
                          alias_path, self.name);
                    return;
                }

                out.insert(alias_path, self.annotations.clone());
                self.annotations = AnnotationSet::new();
            }
            None => { }
        }
    }

    pub fn add_dependencies(&self, library: &Library, out: &mut Dependencies) {
        self.aliased.add_dependencies(library, out);
    }

    pub fn add_monomorphs(&self, library: &Library, out: &mut Monomorphs) {
        self.aliased.add_monomorphs(library, out);
    }

    pub fn mangle_paths(&mut self, monomorphs: &Monomorphs) {
        self.aliased.mangle_paths(monomorphs);
    }
}

impl Source for Typedef {
    fn write<F: Write>(&self, config: &Config, out: &mut SourceWriter<F>) {
        if config.documentation {
            self.documentation.write(config, out);
        }
        out.write("typedef ");
        (self.name.clone(), self.aliased.clone()).write(config, out);
        out.write(";");
    }
}