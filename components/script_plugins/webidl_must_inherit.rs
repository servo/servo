/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use rustc::hir::{self, HirId};
use rustc::lint::{LateContext, LateLintPass, LintArray, LintContext, LintPass};
use rustc::ty;
use std::boxed;
use std::env;
use std::error::Error;
use std::fmt;
use std::fs;
use std::io;
use std::path;
use syntax::ast;
use weedle;

declare_lint!(
    WEBIDL_INHERIT_CORRECT,
    Deny,
    "Warn and report usage of incorrect webidl inheritance"
);

pub(crate) struct WebIdlPass {
    symbols: crate::Symbols,
}

#[derive(Clone, Debug)]
pub struct ParentMismatchError {
    name: String,
    rust_parent: String,
    webidl_parent: String,
}

impl fmt::Display for ParentMismatchError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "webidl-rust inheritance mismatch, rust: {:?}, rust parent: {:?}, webidl parent: {:?}",
            self.name, self.rust_parent, self.webidl_parent
        )
    }
}

impl Error for ParentMismatchError {
    fn description(&self) -> &str {
        "ParentMismatchError"
    }

    fn cause(&self) -> Option<&Error> {
        None
    }
}

impl WebIdlPass {
    pub fn new(symbols: crate::Symbols) -> WebIdlPass {
        WebIdlPass { symbols }
    }
}

fn get_ty_name(ty: &str) -> &str {
    if let Some(i) = ty.rfind(':') {
        &ty[i + 1..]
    } else {
        ty
    }
}

fn get_webidl_path(struct_name: &str) -> io::Result<path::PathBuf> {
    let mut dir = env::current_dir()?;
    dir.push("components/script/dom/webidls/");
    dir.push(format!("{}.webidl", struct_name));

    Ok(dir)
}

fn is_webidl_ty(symbols: &crate::Symbols, cx: &LateContext, ty: &ty::TyS) -> bool {
    let mut ret = false;
    ty.maybe_walk(|t| {
        match t.sty {
            ty::Adt(did, _substs) => {
                if cx.tcx.has_attr(did.did, symbols.webidl) {
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

fn check_inherits(code: &str, name: &str, parent_name: &str) -> Result<(), Box<Error>> {
    let idl = weedle::parse(code).expect("Invalid webidl provided");
    let mut inherits = "";

    for def in idl {
        if let weedle::Definition::Interface(def) = def {
            if let Some(parent) = def.inheritance {
                inherits = parent.identifier.0;
                break;
            }
        } else if let weedle::Definition::CallbackInterface(def) = def {
            if let Some(parent) = def.inheritance {
                inherits = parent.identifier.0;
                break;
            }
        }
    }

    if inherits == parent_name {
        return Ok(());
    }

    // If there is no parent, first field must be of type Reflector.
    if inherits == "" && parent_name == "Reflector" {
        return Ok(());
    }

    if inherits == "" &&
        name == "PaintRenderingContext2D" &&
        parent_name == "CanvasRenderingContext2D"
    {
        // PaintRenderingContext2D embeds a CanvasRenderingContext2D
        // instead of a Reflector as an optimization,
        // but this is fine since CanvasRenderingContext2D
        // also has a reflector
        return Ok(());
    }

    Err(boxed::Box::from(ParentMismatchError {
        name: name.to_string(),
        rust_parent: parent_name.to_string(),
        webidl_parent: inherits.to_string(),
    }))
}

fn check_webidl(name: &str, parent_name: &Option<String>) -> Result<(), Box<Error>> {
    let path = get_webidl_path(&name)?;
    if let Some(parent) = parent_name {
        let code = fs::read_to_string(path)?;
        return check_inherits(&code, name, &parent);
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
        if !is_webidl_ty(&self.symbols, cx, cx.tcx.type_of(def_id)) {
            return;
        }

        let item = match cx.tcx.hir().get_by_hir_id(id) {
            hir::Node::Item(item) => item,
            _ => cx
                .tcx
                .hir()
                .expect_item_by_hir_id(cx.tcx.hir().get_parent_item(id)),
        };

        let parent_name = def.fields().iter().next().map(|field| {
            let def_id = cx.tcx.hir().local_def_id_from_hir_id(field.hir_id);
            let ty = cx.tcx.type_of(def_id).to_string();
            get_ty_name(&ty).to_string()
        });

        let struct_name = n.to_string();
        match check_webidl(&struct_name, &parent_name) {
            Ok(()) => {},
            Err(e) => {
                let description = format!("{}", e);
                cx.span_lint(WEBIDL_INHERIT_CORRECT, item.ident.span, &description)
            },
        };
    }
}
