/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use rustc::lint::Context;
use rustc::middle::{ty, def};

use syntax::ptr::P;
use syntax::{ast, ast_map};
use syntax::ast::{TyPath, Path, AngleBracketedParameters, PathSegment, Ty};
use syntax::attr::mark_used;


/// Matches a type with a provided string, and returns its type parameters if successful
///
/// Try not to use this for types defined in crates you own, use match_lang_ty instead (for lint passes)
pub fn match_ty_unwrap<'a>(ty: &'a Ty, segments: &[&str]) -> Option<&'a [P<Ty>]> {
    match ty.node {
        TyPath(Path {segments: ref seg, ..}, _) => {
            // So ast::Path isn't the full path, just the tokens that were provided.
            // I could muck around with the maps and find the full path
            // however the more efficient way is to simply reverse the iterators and zip them
            // which will compare them in reverse until one of them runs out of segments
            if seg.iter().rev().zip(segments.iter().rev()).all(|(a,b)| a.identifier.as_str() == *b) {
                match seg.as_slice().last() {
                    Some(&PathSegment {parameters: AngleBracketedParameters(ref a), ..}) => {
                        Some(a.types.as_slice())
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
pub fn match_lang_ty(cx: &Context, ty: &Ty, value: &str) -> bool {
    let ty_id = match ty.node {
        TyPath(_, ty_id) => ty_id,
        _ => return false,
    };

    let def_id = match cx.tcx.def_map.borrow().get(&ty_id).cloned() {
        Some(def::DefTy(def_id, _)) => def_id,
        _ => return false,
    };

    ty::get_attrs(cx.tcx, def_id).iter().any(|attr| {
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
            match *itm {
                ast::MethodImplItem(ref meth) => match meth.node {
                    ast::MethDecl(_, _, _, _, style, _, _, _) => match style {
                        ast::Unsafety::Unsafe => true,
                        _ => false,
                    },
                    _ => false,
                },
                _ => false,
            }
        },
        Some(ast_map::NodeItem(itm)) => {
            match itm.node {
                ast::ItemFn(_, style, _, _, _) => match style {
                    ast::Unsafety::Unsafe => true,
                    _ => false,
                },
                _ => false,
            }
        }
        _ => false // There are probably a couple of other unsafe cases we don't care to lint, those will need to be added.
    }
}
