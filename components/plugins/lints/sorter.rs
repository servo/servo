/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use rustc::lint::{Context, LintPass, LintArray};
use std::cmp::Ordering;
use syntax::ast::{Item, Item_, Lit_, MetaItem_, Mod, PathListItem_, ViewPath_, Visibility};
use syntax::codemap::Span;
use syntax::print::pprust::path_to_string;

declare_lint!(UNSORTED_DECLARATIONS, Warn,
              "Warn when the declarations of crates or modules are not in alphabetical order");

pub struct Sorter;

impl LintPass for Sorter {
    fn get_lints(&self) -> LintArray {
        lint_array!(UNSORTED_DECLARATIONS)
    }

    fn check_mod(&mut self, cx: &Context, module: &Mod, _span: Span, _id: u32) {
        let session_codemap = cx.tcx.sess.codemap();    // required only for checking inline mods
        let mut extern_crates = Vec::new();
        let mut uses = Vec::new();
        let mut mods = Vec::new();

        for item in &module.items {
            let item_name = format!("{}", item.ident.name.as_str());
            let item_span = item.span;
            match item.node.clone() {
                Item_::ItemExternCrate(optional_name) if item_name != "std" => {
                    // We've put the declaration here because, we have to sort crate declarations
                    // with respect to the renamed version (instead of the old one).
                    // Since we also don't have `pub` (indicated by the `false` below),
                    // we could just append the declaration to the attributes.
                    let mut item_attrs = get_item_attrs(&item, false);
                    item_attrs = match optional_name {    // for `extern crate foo as bar`
                        Some(old_name) => format!("{}extern crate {} as", item_attrs, old_name),
                        None => format!("{}extern crate", item_attrs),
                    };
                    extern_crates.push((item_name, item_attrs, item_span, false));
                },
                Item_::ItemMod(module) => {
                    let mod_invoked_file = session_codemap.span_to_filename(item.span);
                    let mod_declared_file = session_codemap.span_to_filename(module.inner);
                    if mod_declared_file != mod_invoked_file {      // ignores inline modules
                        let item_attrs = get_item_attrs(&item, true);
                        mods.push((item_name, item_attrs, item_span, false));
                    }
                },
                Item_::ItemUse(spanned) => {
                    let item_attrs = get_item_attrs(&item, true);
                    match spanned.node {
                        ViewPath_::ViewPathSimple(ref ident, ref path) => {
                            let path_str = path_to_string(&path);
                            let name = ident.name.as_str();
                            let renamed = {     // for checking `use foo as bar`
                                let split = path_str.split(":").collect::<Vec<&str>>();
                                match split[split.len() - 1] == &*name {
                                    true => path_str.clone(),
                                    false => format!("{} as {}", &path_str, &name),
                                }
                            };
                            uses.push((renamed, item_attrs, item_span, false));
                        },
                        ViewPath_::ViewPathList(ref path, ref list) => {
                            let old_list = list
                                           .iter()
                                           .map(|&list_item| {
                                                match list_item.node {
                                                    PathListItem_::PathListMod { .. } =>
                                                        "self".to_owned(),      // this must be `self`
                                                    PathListItem_::PathListIdent { name, .. } => {
                                                        // we don't have any renames inside brackets in servo
                                                        let interned = name.name.as_str();
                                                        let string = &*interned;
                                                        string.to_owned()
                                                    },
                                                }
                                            }).collect::<Vec<String>>();
                            let mut new_list = old_list.clone();
                            new_list.sort_by(|a, b| {
                                match (&**a, &**b) {    // `self` should be first in an use list
                                    ("self", _) => Ordering::Less,
                                    (_, "self") => Ordering::Greater,
                                    _ => a.cmp(b),
                                }
                            });
                            let mut warn = false;
                            for i in 0..old_list.len() {
                                if old_list[i] != new_list[i] {
                                    warn = true;    // check whether the use list is sorted
                                    break;
                                }
                            }
                            let use_list = format!("{}::{{{}}}", path_to_string(&path), new_list.join(", "));
                            uses.push((use_list, item_attrs, path.span, warn));
                        },
                        ViewPath_::ViewPathGlob(ref path) => {
                            let path_str = path_to_string(&path) + "::*";
                            // we don't have any use statements like `use std::prelude::*`
                            // since it's done only by rustc, we can safely neglect those here
                            if !path_str.starts_with("std::") {
                                uses.push((path_str, item_attrs, item_span, false));
                            }
                        },
                    }
                },
                _ => (),
            }
        }

        // we don't include the declaration here, because we've already appended it with the attributes
        check_sort(&extern_crates, cx, "crate declarations", "");
        check_sort(&mods, cx, "module declarations (other than inline modules)", "mod");
        check_sort(&uses, cx, "use statements", "use");

        // for collecting, formatting & filtering the attributes (and checking the visibility)
        fn get_item_attrs(item: &Item, pub_check: bool) -> String {
            let mut attr_vec = item.attrs
                               .iter()
                               .filter_map(|attr| {
                                   let meta_item = attr.node.value.node.clone();
                                   let meta_string = get_meta_as_string(&meta_item);
                                   match meta_string.starts_with("doc = ") {
                                       true => None,
                                       false => Some(format!("#[{}]", meta_string)),
                                   }
                               }).collect::<Vec<String>>();
            attr_vec.sort_by(|a, b| {
                match (&**a, &**b) {    // put `macro_use` first for later checking
                    ("#[macro_use]", _) => Ordering::Less,
                    (_, "#[macro_use]") => Ordering::Greater,
                    _ => a.cmp(b),
                }
            });
            let attr_string = attr_vec.join("\n");
            match item.vis {
                Visibility::Public if pub_check => {
                    match attr_string.is_empty() {
                        true => "pub ".to_owned(),
                        false => attr_string + "\npub ",    // `pub` for mods and uses
                    }
                },
                _ => {
                    match attr_string.is_empty() {
                        true => attr_string,
                        false => attr_string + "\n",
                    }
                },
            }
        }

        // collect the information from meta items into Strings
        fn get_meta_as_string(meta_item: &MetaItem_) -> String {
            match *meta_item {
                MetaItem_::MetaWord(ref string) => format!("{}", string),
                MetaItem_::MetaList(ref string, ref meta_items) => {
                    let stuff = meta_items
                                .iter()
                                .map(|meta_item| {
                                    get_meta_as_string(&meta_item.node)
                                }).collect::<Vec<String>>();
                    format!("{}({})", string, stuff.join(", "))
                },
                MetaItem_::MetaNameValue(ref string, ref literal) => {
                    let value = match literal.node {
                        Lit_::LitStr(ref inner_str, _style) => inner_str,
                        _ => panic!("unexpected literal found for meta item!"),
                    }; format!("{} = \"{}\"", string, value)
                },
            }
        }

        // checks the sorting of all the declarations and raises warnings whenever necessary
        // takes a slice of tuples with name, related attributes, spans and whether to warn for unordered use lists
        fn check_sort(old_list: &[(String, String, Span, bool)], cx: &Context, kind: &str, syntax: &str) {
            let length = old_list.len();
            let mut new_list = old_list
                                .iter()
                                .map(|&(ref name, ref attrs, _span, warn)| (name.clone(), attrs.clone(), warn))
                                .collect::<Vec<(String, String, bool)>>();
            new_list.sort_by(|&(ref str_a, ref attr_a, _), &(ref str_b, ref attr_b, _)| {
                // move the `pub` statements below (with `~` since it's on the farther side of ASCII)
                let mut new_str_a = str_for_biased_sort(&str_a, attr_a.ends_with("pub "), "~");
                let mut new_str_b = str_for_biased_sort(&str_b, attr_b.ends_with("pub "), "~");
                // move the #[macro_use] stuff above (with `!` since it's on the lower extreme of ASCII)
                new_str_a = str_for_biased_sort(&new_str_a, attr_a.starts_with("#[macro_use]"), "!");
                new_str_b = str_for_biased_sort(&new_str_b, attr_b.starts_with("#[macro_use]"), "!");
                new_str_a.cmp(&new_str_b)
            });

            let mut index = 0;
            let mut span: Option<Span> = None;
            for i in 0..length {
                if (old_list[i].0 != new_list[i].0) || new_list[i].2 {
                    span = Some(old_list[i].2);
                    index = i;      // only to find the index of the first unsorted declaration
                    break;          // because, we'll be printing everything following the first unsorted one
                }
            }

            match span {
                Some(span_start) => {   // print all the declarations proceeding the first unsorted one
                    let suggestion_list = (index..length)
                                          .map(|i| {
                                              if i == length - 1 {  // increase the span to include more lines
                                                  let mut sp = span_start;
                                                  sp.hi = old_list[i].2.hi;
                                                  span = Some(sp);
                                              } format!("{}{} {};", new_list[i].1, syntax, new_list[i].0)
                                          }).collect::<Vec<String>>();
                    let suggestion = format!("{} should be in alphabetical order!\nTry this...\n\n{}\n\n",
                                            kind, suggestion_list.join("\n"));
                    // unwrapping the value here, because it's quite certain that there's something in `span`
                    cx.span_lint(UNSORTED_DECLARATIONS, span.unwrap(), &suggestion);
                },
                None => (),
            }

            // prepend given characters to names for biased sorting
            fn str_for_biased_sort(string: &String, choice: bool, prepend_char: &str) -> String {
                match choice {
                    true => prepend_char.to_owned() + &**string,
                    false => string.clone()
                }
            }
        }
    }
}
