/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use rustc::hir::{self, HirId};
use rustc::lint::{LateContext, LateLintPass, LintArray, LintContext, LintPass};
use rustc::ty;
use std::env;

use std::io;
use std::path;
use syntax::ast;
declare_lint!(
    WEBIDL_INHERIT_CORRECT,
    Deny,
    "Warn and report usage of incorrect webidl inheritance"
);

pub struct WebIdlPass;

impl WebIdlPass {
    pub fn new() -> WebIdlPass {
        WebIdlPass
    }
}

fn get_typ_name(typ: String) -> Option<String> {
    if let Some(i) = typ.rfind(':') {
        return Some(typ[i + 1..].to_string());
    }
    None
}

fn get_webidl_path(struct_name: &str) -> io::Result<path::PathBuf> {
    let mut dir = env::current_dir()?;
    dir.push("components/script/dom/webidls/");
    dir.push(format!("{}.webidl", struct_name));

    return Ok(dir);
}

/// Checks if a type is unrooted or contains any owned unrooted types
fn is_webidl_ty(cx: &LateContext, ty: &ty::TyS) -> bool {
    let mut ret = false;
    ty.maybe_walk(|t| {
        match t.sty {
            ty::Adt(did, _substs) => {
                if cx.tcx.has_attr(did.did, "webidl") {
                    ret = true;
                }
                false
            },
            ty::Ref(..) => false,    // don't recurse down &ptrs
            ty::RawPtr(..) => false, // don't recurse down *ptrs
            ty::FnDef(..) | ty::FnPtr(_) => false,
            _ => true,
        }
    });
    ret
}

fn check_webidl(name: &str, parent_name: Option<String>) -> io::Result<()> {
    let path = get_webidl_path(&name)?;
    println!("struct_webidl_path: {:?}", &path);

    if let Some(parent) = parent_name {
        let parent_path = get_webidl_path(&parent)?;
        println!("parent_path: {:?}", &parent_path);
    }

    Ok(())
}

impl LintPass for WebIdlPass {
    fn name(&self) -> &'static str {
        "ServoWebIDLPass"
    }

    fn get_lints(&self) -> LintArray {
        lint_array!(WEBIDL_INHERIT_CORRECT)
    }
}

impl<'a, 'tcx> LateLintPass<'a, 'tcx> for WebIdlPass {
    fn check_struct_def(
        &mut self,
        cx: &LateContext<'a, 'tcx>,
        def: &'tcx hir::VariantData,
        n: ast::Name,
        _gen: &'tcx hir::Generics,
        id: HirId,
    ) {
        let def_id = cx.tcx.hir().local_def_id_from_hir_id(id);
        if !is_webidl_ty(cx, cx.tcx.type_of(def_id)) {
            return;
        }

        let item = match cx.tcx.hir().get_by_hir_id(id) {
            hir::Node::Item(item) => item,
            _ => cx
                .tcx
                .hir()
                .expect_item_by_hir_id(cx.tcx.hir().get_parent_item(id)),
        };

        let struct_name = n.to_string();
        println!("struct_name: {:?}", struct_name);

        let mut parent_typ_name: Option<String> = None;
        for ref field in def.fields() {
            let def_id = cx.tcx.hir().local_def_id_from_hir_id(field.hir_id);
            let ty = cx.tcx.type_of(def_id);
            let typ = ty.to_string();
            if let Some(typ_name) = get_typ_name(ty.to_string()) {
                parent_typ_name = Some(typ_name);
                break;
            } else {
                cx.span_lint(WEBIDL_INHERIT_CORRECT, field.span, "Cannot get type name");
            }

            // Only first field is relevant.
            break;
        }

        // TODO Open and parse corresponding webidl file.
        cx.span_lint(WEBIDL_INHERIT_CORRECT, item.ident.span, "WEBIDL present.");

        match check_webidl(&struct_name, parent_typ_name) {
            Ok(()) => {},
            Err(e) => println!("ERRORRR: {:?}", e),
        };
    }
}
