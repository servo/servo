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
use webidl::ast::*;
use webidl::visitor::*;
use webidl::*;

declare_lint!(
    WEBIDL_INHERIT_CORRECT,
    Deny,
    "Warn and report usage of incorrect webidl inheritance"
);

pub struct WebIdlPass;

#[derive(Clone, Debug)]
pub enum WebIdlError {
    ParentMismatch {
        name: String,
        rust_parent: String,
        webidl_parent: String,
    },
}

impl fmt::Display for WebIdlError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            WebIdlError::ParentMismatch {
                name,
                rust_parent,
                webidl_parent,
            } => {
                return write!(f, "webidl-rust inheritance mismatch, rust: {:?}, rust parent: {:?}, webidl parent: {:?}",
                    &name, &rust_parent, &webidl_parent);
            },
        }
    }
}

impl Error for WebIdlError {
    fn description(&self) -> &str {
        "WebIdlError"
    }

    fn cause(&self) -> Option<&Error> {
        None
    }
}

impl WebIdlPass {
    pub fn new() -> WebIdlPass {
        WebIdlPass
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

fn check_inherits(code: &str, name: &str, parent_name: &str) -> Result<(), Box<Error>> {
    let idl = parse_string(code)?;
    let mut visitor = InterfaceVisitor::new(name.to_string());
    visitor.visit(&idl);
    let inherits = visitor.get_inherits();

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

    Err(boxed::Box::from(WebIdlError::ParentMismatch {
        name: name.to_string(),
        rust_parent: parent_name.to_string(),
        webidl_parent: inherits.to_string(),
    }))
}

fn check_webidl(name: &str, parent_name: Option<&str>) -> Result<(), Box<Error>> {
    let path = get_webidl_path(&name)?;
    if let Some(parent) = parent_name {
        let code = fs::read_to_string(path)?;
        return check_inherits(&code, &name, parent);
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

        let ty: String;
        let mut parent_name: Option<&str> = None;
        for ref field in def.fields() {
            let def_id = cx.tcx.hir().local_def_id_from_hir_id(field.hir_id);
            ty = cx.tcx.type_of(def_id).to_string();
            let name = get_ty_name(&ty);
            parent_name = Some(name);

            // Only first field is relevant.
            break;
        }

        let struct_name = n.to_string();
        match check_webidl(&struct_name, parent_name) {
            Ok(()) => {},
            Err(e) => {
                let description = format!("{}", e);
                cx.span_lint(WEBIDL_INHERIT_CORRECT, item.ident.span, &description)
            },
        };
    }
}

struct InterfaceVisitor {
    name: String,
    inherits: String,
}

impl InterfaceVisitor {
    pub fn new(name: String) -> Self {
        InterfaceVisitor {
            name: name,
            inherits: String::new(),
        }
    }

    pub fn get_inherits(&self) -> &String {
        &self.inherits
    }
}

impl<'ast> ImmutableVisitor<'ast> for InterfaceVisitor {
    fn visit_callback_interface(&mut self, callback_interface: &'ast CallbackInterface) {
        if callback_interface.name == self.name {
            if let Some(ref inherit) = callback_interface.inherits {
                self.inherits = inherit.to_string()
            }
        }
    }

    fn visit_non_partial_interface(&mut self, non_partial_interface: &'ast NonPartialInterface) {
        if non_partial_interface.name == self.name {
            if let Some(ref inherit) = non_partial_interface.inherits {
                self.inherits = inherit.to_string()
            }
        }
    }
}
