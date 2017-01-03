/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use rustc::hir::{self, def};
use rustc::hir::def_id::DefId;
use rustc::lint::{LateContext, LintContext};
use syntax::ast;
use syntax::attr::mark_used;
use syntax::codemap::{ExpnFormat, Span};
use syntax::ptr::P;


/// Matches a type with a provided string, and returns its type parameters if successful
///
/// Try not to use this for types defined in crates you own, use match_lang_ty instead (for lint passes)
pub fn match_ty_unwrap<'a>(ty: &'a ast::Ty, segments: &[&str]) -> Option<&'a [P<ast::Ty>]> {
    match ty.node {
        ast::TyKind::Path(_, ast::Path { segments: ref seg, .. }) => {
            // So hir::Path isn't the full path, just the tokens that were provided.
            // I could muck around with the maps and find the full path
            // however the more efficient way is to simply reverse the iterators and zip them
            // which will compare them in reverse until one of them runs out of segments
            if seg.iter().rev().zip(segments.iter().rev()).all(|(a, b)| &*a.identifier.name.as_str() == *b) {
                match seg.last() {
                    Some(&ast::PathSegment { parameters: Some(ref params), .. }) => {
                        match **params {
                            ast::PathParameters::AngleBracketed(ref a) => Some(&a.types),

                            // `Foo(A,B) -> C`
                            ast::PathParameters::Parenthesized(_) => None,
                        }
                    }
                    Some(&ast::PathSegment { parameters: None, .. }) => Some(&[]),
                    None => None,
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
    let def = match ty.node {
        hir::TyPath(hir::QPath::Resolved(_, ref path)) => path.def,
        _ => return false,
    };

    if let def::Def::PrimTy(_) = def {
        return false;
    }

    match_lang_did(cx, def.def_id(), value)
}

pub fn match_lang_did(cx: &LateContext, did: DefId, value: &str) -> bool {
    cx.tcx.get_attrs(did).iter().any(|attr| {
        if attr.check_name("servo_lang") && attr.value_str().map_or(false, |v| v == value) {
            mark_used(attr);
            true
        } else {
            false
        }
    })
}

/// check if a DefId's path matches the given absolute type path
/// usage e.g. with
/// `match_def_path(cx, id, &["core", "option", "Option"])`
pub fn match_def_path(cx: &LateContext, def_id: DefId, path: &[&str]) -> bool {
    let krate = &cx.tcx.crate_name(def_id.krate);
    if krate != &path[0] {
        return false;
    }

    let path = &path[1..];
    let other = cx.tcx.def_path(def_id).data;

    if other.len() != path.len() {
        return false;
    }

    other.into_iter()
         .map(|e| e.data)
         .zip(path)
         .all(|(nm, p)| &*nm.as_interned_str() == *p)
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
