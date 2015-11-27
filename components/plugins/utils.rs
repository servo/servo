/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use rustc::front::map as ast_map;
use rustc::lint::{LateContext, LintContext};
use rustc::middle::def;
use rustc::middle::def_id::DefId;
use rustc_front::hir;
use syntax::ast;
use syntax::attr::mark_used;
use syntax::codemap::{ExpnFormat, Span};
use syntax::ptr::P;


/// Matches a type with a provided string, and returns its type parameters if successful
///
/// Try not to use this for types defined in crates you own, use match_lang_ty instead (for lint passes)
pub fn match_ty_unwrap<'a>(ty: &'a ast::Ty, segments: &[&str]) -> Option<&'a [P<ast::Ty>]> {
    match ty.node {
        ast::TyPath(_, ast::Path { segments: ref seg, .. }) => {
            // So hir::Path isn't the full path, just the tokens that were provided.
            // I could muck around with the maps and find the full path
            // however the more efficient way is to simply reverse the iterators and zip them
            // which will compare them in reverse until one of them runs out of segments
            if seg.iter().rev().zip(segments.iter().rev()).all(|(a, b)| a.identifier.name.as_str() == *b) {
                match seg.last() {
                    Some(&ast::PathSegment { parameters: ast::AngleBracketedParameters(ref a), .. }) => {
                        Some(&a.types)
                    }
                    _ => None
                }
            } else {
                None
            }
        },
        _ => None
    }
}

/// Checks if a type has a #[servo_lang = "str"] attribute
pub fn match_lang_ty(cx: &LateContext, ty: &hir::Ty, value: &str) -> bool {
    match ty.node {
        hir::TyPath(..) => {},
        _ => return false,
    }

    let def_id = match cx.tcx.def_map.borrow().get(&ty.id) {
        Some(&def::PathResolution { base_def: def::DefTy(def_id, _), .. }) => def_id,
        _ => return false,
    };

    match_lang_did(cx, def_id, value)
}

pub fn match_lang_did(cx: &LateContext, did: DefId, value: &str) -> bool {
    cx.tcx.get_attrs(did).iter().any(|attr| {
        match attr.node.value.node {
            ast::MetaNameValue(ref name, ref val) if &**name == "servo_lang" => {
                match val.node {
                    ast::LitStr(ref v, _) if &**v == value => {
                        mark_used(attr);
                        true
                    },
                    _ => false,
                }
            }
            _ => false,
        }
    })
}

// Determines if a block is in an unsafe context so that an unhelpful
// lint can be aborted.
pub fn unsafe_context(map: &ast_map::Map, id: ast::NodeId) -> bool {
    match map.find(map.get_parent(id)) {
        Some(ast_map::NodeImplItem(itm)) => {
            match itm.node {
                hir::ImplItemKind::Method(ref sig, _) => sig.unsafety == hir::Unsafety::Unsafe,
                _ => false
            }
        },
        Some(ast_map::NodeItem(itm)) => {
            match itm.node {
                hir::ItemFn(_, style, _, _, _, _) => match style {
                    hir::Unsafety::Unsafe => true,
                    _ => false,
                },
                _ => false,
            }
        }
        _ => false // There are probably a couple of other unsafe cases we don't care to lint, those will need
                   // to be added.
    }
}

/// check if a DefId's path matches the given absolute type path
/// usage e.g. with
/// `match_def_path(cx, id, &["core", "option", "Option"])`
pub fn match_def_path(cx: &LateContext, def_id: DefId, path: &[&str]) -> bool {
    cx.tcx.with_path(def_id, |iter| iter.map(|elem| elem.name())
        .zip(path.iter()).all(|(nm, p)| &nm.as_str() == p))
}

pub fn in_derive_expn(cx: &LateContext, span: Span) -> bool {
    cx.sess().codemap().with_expn_info(span.expn_id,
            |info| {
                if let Some(i) = info {
                    if let ExpnFormat::MacroAttribute(n) = i.callee.format {
                        if n.as_str().contains("derive") {
                            true
                        } else { false }
                    } else { false }
                } else { false }
            })
}
