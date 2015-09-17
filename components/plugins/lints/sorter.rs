/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use rustc::lint::{Context, LintPass, LintArray};
use std::cmp::Ordering;
use std::fmt::Display;
use syntax::ast::{Mod, Item_, PathListItem_, ViewPath_};
use syntax::codemap::{Pos, BytePos, Span};
use syntax::print::pprust::path_to_string;

declare_lint!(UNSORTED_DECLARATIONS, Deny,
              "Warn when the declarations of crates or modules are not in alphabetical order");

pub struct Sorter;

impl LintPass for Sorter {
    fn get_lints(&self) -> LintArray {
        lint_array!(UNSORTED_DECLARATIONS)
    }

    fn check_mod(&mut self, cx: &Context, module: &Mod, _span: Span, _id: u32) {
        let session_codemap = cx.tcx.sess.codemap();
        let mut extern_crates = Vec::new();
        let mut uses = Vec::new();
        let mut mods = Vec::new();
        for item in &module.items {
            let item_name = item.ident.name.as_str();
            let item_span = item.span;
            match item.node.clone() {
                Item_::ItemExternCrate(_) if item_name != "std" => {
                    extern_crates.push((item_name, item_span));
                },
                Item_::ItemMod(module) => {
                    let mod_invoked_file = session_codemap.span_to_filename(item.span);
                    let mod_declared_file = session_codemap.span_to_filename(module.inner);
                    if mod_declared_file != mod_invoked_file {      // this ignores inline modules
                        mods.push((item_name, item_span));
                    }
                },
                Item_::ItemUse(spanned) => {
                    match spanned.node {
                        ViewPath_::ViewPathSimple(_, ref path) => {
                            uses.push((path_to_string(&path), item_span));
                        },
                        ViewPath_::ViewPathList(ref path, ref list) => {
                            let old_list = list
                                           .iter()
                                           .filter_map(|&list_item| {
                                                match list_item.node {
                                                    PathListItem_::PathListIdent { name, .. } => {
                                                        let interned = name.name.as_str();
                                                        let string = &*interned;
                                                        Some(string.to_owned())
                                                    },
                                                    _ => None,
                                                }
                                            }).collect::<Vec<String>>();
                            let mut new_list = old_list.clone();
                            new_list.sort_by(|a, b| {
                                match *a == "self" {
                                    true => Ordering::Less,
                                    false => a.cmp(b),
                                }
                            });
                            for i in 0..old_list.len() {
                                if old_list[i] != new_list[i] {
                                    let suggestion = format!("use lists should be in alphabetical order!\
                                                              \n\texpected order of list: {{{}}}\
                                                              \n\tlist found: {{{}}}",
                                                              new_list.join(", "), old_list.join(", "));
                                    cx.span_lint(UNSORTED_DECLARATIONS, path.span, &suggestion);
                                    break;
                                }
                            }
                        },
                        ViewPath_::ViewPathGlob(ref path) => {
                            let path_str = path_to_string(&path);
                            // we don't have any use statements like `use std::prelude::*`
                            // since it's done only by rustc, we can safely neglect those here
                            if !path_str.starts_with("std::") {
                                uses.push((path_str, item_span));
                            }
                        },
                    }
                },
                _ => (),
            }
        }

        check_sort(&extern_crates, cx, "crate declarations", 12);   // assert_eq!("extern crate".len() == 12, true)
        check_sort(&mods, cx, "module declarations", 3);
        check_sort(&uses, cx, "use statements", 3);

        fn check_sort<T: Ord + PartialEq + Clone + Display>(old_slice: &[(T, Span)],
                                                            cx: &Context,
                                                            kind: &str,
                                                            syntax_len: usize) {
            let mut new_slice = old_slice
                                .iter()
                                .map(|&(ref string, _span)| string.clone())
                                .collect::<Vec<T>>();
            new_slice.sort();
            for i in 0..old_slice.len() {
                let (name, mut span) = (old_slice[i].0.clone(), old_slice[i].1);
                span.lo = span.lo + BytePos::from_usize(syntax_len + 1);
                if name != new_slice[i] {
                    let suggestion = format!("{} should be in alphabetical order!\n\t\
                                              expected: {}\n\tfound: {}",
                                              kind, new_slice[i], name);
                    cx.span_lint(UNSORTED_DECLARATIONS, span, &suggestion);
                }
            }
        }
    }
}
