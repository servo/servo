/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use rustc::lint::{Context, LintPass, LintArray};
use rustc::middle::def_id::DefId;
use syntax::ast;
use syntax::ast::Public;
use syntax::attr::AttrMetaMethods;

declare_lint!(PRIVATIZE, Deny,
              "Allows to enforce private fields for struct definitions");

/// Lint for keeping DOM fields private
///
/// This lint (disable with `-A privatize`/`#[allow(privatize)]`) ensures all types marked with `#[privatize]`
/// have no public fields
pub struct PrivatizePass;

impl LintPass for PrivatizePass {
    fn get_lints(&self) -> LintArray {
        lint_array!(PRIVATIZE)
    }

    fn check_struct_def(&mut self,
                        cx: &Context,
                        def: &ast::StructDef,
                        _i: ast::Ident,
                        _gen: &ast::Generics,
                        id: ast::NodeId) {
        if cx.tcx.has_attr(DefId::local(id), "privatize") {
            for field in &def.fields {
                match field.node {
                    ast::StructField_ { kind: ast::NamedField(ident, visibility), .. } if visibility == Public => {
                        cx.span_lint(PRIVATIZE, field.span,
                                     &format!("Field {} is public where only private fields are allowed",
                                              ident.name));
                    }
                    _ => {}
                }
            }
        }
    }
}
