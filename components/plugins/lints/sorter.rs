/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use rustc::lint::{Context, LintPass, LintArray};
use syntax::ast::{Crate, Mod, Item_, ViewPath_};
use syntax::codemap::Span;
use syntax::print::pprust::path_to_string;

declare_lint!(SORTER, Deny,
              "Warn when the declarations of crates or modules are not in alphabetical order");

pub struct Sorter;

impl LintPass for Sorter {
    fn get_lints(&self) -> LintArray {
        lint_array!(SORTER)
    }

    fn check_mod(&mut self, _cx: &Context, module: &Mod, _span: Span, _id: u32) {
        let mut extern_crates = Vec::new();
        let mut uses = Vec::new();
        // let mut mods = Vec::new();
        for item in &module.items {
            match item.node.clone() {
                Item_::ItemExternCrate(_) if item.ident.name.as_str() != "std" => {
                    extern_crates.push(item.ident.name.as_str());
                },
                Item_::ItemUse(spanned) => {
                    match spanned.node {
                        ViewPath_::ViewPathSimple(_, ref path) | ViewPath_::ViewPathList(ref path, _) => {
                            uses.push(path_to_string(&path));
                        },
                        ViewPath_::ViewPathGlob(ref path) => {
                            let path_str = path_to_string(&path);
                            // we don't have any use statements like `use std::<something>::*`
                            // since it's done only by rustc, we can safely neglect those here
                            match path_str.starts_with("std::") {
                                true => continue,
                                false => uses.push(path_str),
                            }
                        },
                    }
                },
                _ => {},
            }
        }
    }
}
