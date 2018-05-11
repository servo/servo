/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use rustc::hir;
use rustc::hir::intravisit as visit;
use rustc::hir::map as ast_map;
use rustc::lint::{LateContext, LintPass, LintArray, LateLintPass, LintContext};
use rustc::mir;
use rustc::mir::visit::Visitor as MirVisitor;
use rustc::ty;
use syntax::{ast, codemap};
use utils::{match_def_path, in_derive_expn, get_def_path};

declare_lint!(UNROOTED_MUST_ROOT, Deny,
              "Warn and report usage of unrooted jsmanaged objects");

/// Lint for ensuring safe usage of unrooted pointers
///
/// This lint (disable with `-A unrooted-must-root`/`#[allow(unrooted_must_root)]`) ensures that `#[must_root]`
/// values are used correctly.
///
/// "Incorrect" usage includes:
///
///  - Not being used in a struct/enum field which is not `#[must_root]` itself
///  - Not being used as an argument to a function (Except onces named `new` and `new_inherited`)
///  - Not being bound locally in a `let` statement, assignment, `for` loop, or `match` statement.
///
/// This helps catch most situations where pointers like `JS<T>` are used in a way that they can be invalidated by a
/// GC pass.
///
/// Structs which have their own mechanism of rooting their unrooted contents (e.g. `ScriptThread`)
/// can be marked as `#[allow(unrooted_must_root)]`. Smart pointers which root their interior type
/// can be marked as `#[allow_unrooted_interior]`
pub struct UnrootedPass;

impl UnrootedPass {
    pub fn new() -> UnrootedPass {
        UnrootedPass
    }
}

impl LintPass for UnrootedPass {
    fn get_lints(&self) -> LintArray {
        lint_array!(UNROOTED_MUST_ROOT)
    }
}

impl<'a, 'tcx> LateLintPass<'a, 'tcx> for UnrootedPass {
    /// All structs containing #[must_root] types must be #[must_root] themselves
    fn check_struct_def(&mut self,
                        cx: &LateContext,
                        def: &hir::VariantData,
                        _n: ast::Name,
                        _gen: &hir::Generics,
                        id: ast::NodeId) {
        let unrooted_cx = UnrootedCx {
            late_cx: cx,
            def_id: cx.tcx.hir.local_def_id(id),
        };
        let item = match cx.tcx.hir.get(id) {
            ast_map::Node::NodeItem(item) => item,
            _ => cx.tcx.hir.expect_item(cx.tcx.hir.get_parent(id)),
        };
        if item.attrs.iter().all(|a| !a.check_name("must_root")) {
            for ref field in def.fields() {
                let def_id = cx.tcx.hir.local_def_id(field.id);
                if unrooted_cx.is_unrooted_ty(cx.tcx.type_of(def_id), false) {
                    cx.span_lint(UNROOTED_MUST_ROOT, field.span,
                                 "Type must be rooted, use #[must_root] on the struct definition to propagate")
                }
            }
        }
    }

    /// All enums containing #[must_root] types must be #[must_root] themselves
    fn check_variant(&mut self, cx: &LateContext, var: &hir::Variant, _gen: &hir::Generics) {
        let ref map = cx.tcx.hir;
        let parent_node = map.get_parent(var.node.data.id());
        let unrooted_cx = UnrootedCx {
            late_cx: cx,
            def_id: map.local_def_id(parent_node),
        };
        if map.expect_item(parent_node).attrs.iter().all(|a| !a.check_name("must_root")) {
            match var.node.data {
                hir::VariantData::Tuple(ref fields, _) => {
                    for ref field in fields {
                        let def_id = cx.tcx.hir.local_def_id(field.id);
                        if unrooted_cx.is_unrooted_ty(cx.tcx.type_of(def_id), false) {
                            cx.span_lint(UNROOTED_MUST_ROOT, field.ty.span,
                                         "Type must be rooted, use #[must_root] on \
                                          the enum definition to propagate")
                        }
                    }
                }
                _ => () // Struct variants already caught by check_struct_def
            }
        }
    }

    /// Function arguments that are #[must_root] types are not allowed
    fn check_fn(&mut self,
                cx: &LateContext<'a, 'tcx>,
                kind: visit::FnKind,
                _decl: &'tcx hir::FnDecl,
                _body: &'tcx hir::Body,
                span: codemap::Span,
                id: ast::NodeId) {
        let in_new_function = match kind {
            visit::FnKind::ItemFn(n, _, _, _, _, _, _) |
            visit::FnKind::Method(n, _, _, _) => {
                &*n.as_str() == "new" || n.as_str().starts_with("new_")
            }
            visit::FnKind::Closure(_) => return,
        };

        let def_id = cx.tcx.hir.local_def_id(id);
        let mir = cx.tcx.optimized_mir(def_id);
        let unrooted_cx = UnrootedCx {
            late_cx: cx,
            def_id: def_id,
        };
        let mut visitor = MirFnVisitor {
            unrooted_cx: unrooted_cx,
            in_new_function: in_new_function,
            in_derive_expn: in_derive_expn(span),  // TODO replace with whitelist
            mir: mir,
        };

        visitor.visit_mir(mir);
    }
}

struct UnrootedCx<'a, 'b: 'a, 'tcx: 'a + 'b> {
    late_cx: &'a LateContext<'b, 'tcx>,

    /// context of definition we want to check
    def_id: hir::def_id::DefId,
}

struct MirFnVisitor<'a, 'b: 'a, 'tcx: 'a + 'b> {
    unrooted_cx: UnrootedCx<'a, 'b, 'tcx>,
    in_new_function: bool,
    in_derive_expn: bool,
    mir: &'a mir::Mir<'tcx>,
}

/// Checks if a type is unrooted or contains any owned unrooted types
impl<'a, 'b, 'tcx> UnrootedCx<'a, 'b, 'tcx> {
    fn is_unrooted_ty(&self, ty: &ty::TyS, in_new_function: bool) -> bool {
        let mut ret = false;
        let cx = self.late_cx;
        ty.maybe_walk(|t| {
            match t.sty {
                ty::TyAdt(did, _) => {
                    if cx.tcx.has_attr(did.did, "must_root") {
                        ret = true;
                        false
                    } else if cx.tcx.has_attr(did.did, "allow_unrooted_interior") {
                        false
                    //} else if match_def_path(cx, did.did, &["mozjs", "conversions", "ToJSValConvertible"]) { // hadn't helped :(
                    //    ret = true;
                    //    false
                    } else if self.is_ptr_like_structure(did.did) {
                        false
                    } else if did.is_box() && in_new_function {
                        // box in new() is okay
                        false
                    } else {
                        true
                    }
                },
                ty::TyParam(param_ty) => {
                    let ty_param_def = cx.tcx.generics_of(self.def_id).type_param(&param_ty, cx.tcx);
                    if cx.tcx.has_attr(ty_param_def.def_id, "must_root") {
                        ret = true;
                        false
                    } else {
                        true
                    }
                },
                ty::TyRef(..) => false, // don't recurse down &ptrs
                ty::TyRawPtr(..) => false, // don't recurse down *ptrs
                ty::TyFnDef(..) | ty::TyFnPtr(_) => false,
                _ => true
            }
        });
        ret
    }

    fn has_unrooted_generic_substs(&self, did: hir::def_id::DefId, substs: &ty::subst::Substs, st: &mut String) -> bool {
        let cx = self.late_cx;

        if cx.tcx.has_attr(did, "allow_unrooted_interior") {
            return false;
        }
        else if self.is_ptr_like_structure(did) {
            return false;
        }

        let def_path = get_def_path(cx, did);
        st.push_str(&" $$ ");
        st.push_str(&def_path.join("::"));
        st.push_str(&"\n");

        let generics = cx.tcx.generics_of(did);
        for ty_param_def in &generics.types {
            // If type has `#[must_root]`, then it is ok to
            // give it a must-root type, so just skip.
            if cx.tcx.has_attr(ty_param_def.def_id, "must_root") {
                continue;
            }

            let arg_ty = substs.type_at(ty_param_def.index as usize);
            if self.is_unrooted_ty(arg_ty, false) {
                st.push_str(&" $$ > ");
                st.push_str(&arg_ty.to_string());
                return true;
            }
        }

        st.push_str(&" par");
        match generics.parent {
            Some(p_did) => self.has_unrooted_generic_substs(p_did, substs, st),
            None => false,
        }
    }

    fn is_ptr_like_structure(&self, did: hir::def_id::DefId) -> bool {
        // Structures which are semantically similar to an &ptr.

        // ------------------- TODO --------------------
        // (1) Not sure if this list should be same for is_unrooted_ty and
        // has_unrooted_generic_substs
        // (2) All entries below with comment (TODO/is ok?) should be checked if they really should
        // be on this list. Currently it is used to mute existing violations and let servo compile.
        // (3) Some ideas what can be done: in is_unrooted_ty some types from outside script crate
        // can be treated always as #[must_root]. For types like Option<T> could be implemented
        // implicit treating it as #[must_root] if and only if T is #[must_root]
        // (auto-propagation).
        let cx = self.late_cx;
        match_def_path(cx, did, &["alloc", "boxed", "Box"])  // -- is ok? // TODO uncomment unit test "ban_box"
        || match_def_path(cx, did, &["alloc", "boxed", "{{impl}}"])  // -- is ok?
        || match_def_path(cx, did, &["alloc", "rc", "Rc"])  // -- is ok?
        || match_def_path(cx, did, &["alloc", "slice", "{{impl}}"])  // -- is ok?
        || match_def_path(cx, did, &["alloc", "vec", "Drain"])  // -- is ok?
        || match_def_path(cx, did, &["alloc", "vec", "Vec"])  // -- is ok?
        || match_def_path(cx, did, &["alloc", "vec", "{{impl}}"])  // -- is ok?
        || match_def_path(cx, did, &["alloc", "vec_deque", "IterMut"])  // -- is ok?
        || match_def_path(cx, did, &["alloc", "vec_deque", "VecDeque"])  // -- is ok?
        || match_def_path(cx, did, &["alloc", "vec_deque", "{{impl}}"]) // -- TODO check if ok **autogenerated**
        || match_def_path(cx, did, &["core", "cell", "Ref"])
        || match_def_path(cx, did, &["core", "cell", "RefCell"])  // -- is ok?
        || match_def_path(cx, did, &["core", "cell", "RefMut"])
        || match_def_path(cx, did, &["core", "cell", "UnsafeCell"])  // -- is ok?
        || match_def_path(cx, did, &["core", "cell", "{{impl}}", "map"]) // -- TODO check if ok **autogenerated**
        || match_def_path(cx, did, &["core", "cell", "{{impl}}"])  // -- is ok?
        || match_def_path(cx, did, &["core", "clone", "Clone"])  // -- is ok?
        || match_def_path(cx, did, &["core", "cmp", "PartialEq"])  // -- is ok?
        || match_def_path(cx, did, &["core", "cmp", "PartialOrd"]) // -- TODO check if ok **autogenerated**
        || match_def_path(cx, did, &["core", "default", "Default"])  // -- is ok?
        || match_def_path(cx, did, &["core", "fmt", "Debug"]) // -- TODO check if ok **autogenerated**
        || match_def_path(cx, did, &["core", "fmt", "Pointer"]) // -- TODO check if ok **autogenerated**
        || match_def_path(cx, did, &["core", "intrinsics", "", "type_name"])  // -- is ok?
        || match_def_path(cx, did, &["core", "iter", "Map"])  // -- is ok?
        || match_def_path(cx, did, &["core", "iter", "iterator", "Iterator", "cloned"]) // -- TODO check if ok **autogenerated**
        || match_def_path(cx, did, &["core", "iter", "iterator", "Iterator", "collect"]) // -- TODO check if ok **autogenerated**
        || match_def_path(cx, did, &["core", "iter", "iterator", "Iterator", "filter_map"]) // -- TODO check if ok **autogenerated**
        || match_def_path(cx, did, &["core", "iter", "iterator", "Iterator", "map"]) // -- TODO check if ok **autogenerated**
        || match_def_path(cx, did, &["core", "iter", "iterator", "Iterator"]) // -- TODO check if ok **autogenerated**
        || match_def_path(cx, did, &["core", "iter", "traits", "Extend"]) // -- TODO check if ok **autogenerated**
        || match_def_path(cx, did, &["core", "iter", "traits", "FromIterator"]) // -- TODO check if ok **autogenerated**
        || match_def_path(cx, did, &["core", "marker", "PhantomData"])  // -- is ok?
        || match_def_path(cx, did, &["core", "mem", "drop"])  // -- is ok?
        || match_def_path(cx, did, &["core", "mem", "size_of"])  // -- is ok?
        || match_def_path(cx, did, &["core", "mem", "swap"]) // -- TODO check if ok **autogenerated**
        || match_def_path(cx, did, &["core", "ops", "deref", "Deref"])  // -- is ok? or only the deref method?
        || match_def_path(cx, did, &["core", "ops", "deref", "DerefMut"]) // -- TODO check if ok **autogenerated**
        || match_def_path(cx, did, &["core", "ops", "index", "Index"])  // -- is ok?
        || match_def_path(cx, did, &["core", "ops", "index", "IndexMut"]) // -- TODO check if ok **autogenerated**
        || match_def_path(cx, did, &["core", "ops", "try", "Try"]) // -- TODO check if ok **autogenerated**
        || match_def_path(cx, did, &["core", "option", "Option"])  // -- is ok? looks like misuse
        || match_def_path(cx, did, &["core", "option", "{{impl}}", "map"])  // -- is ok?
        || match_def_path(cx, did, &["core", "option", "{{impl}}", "unwrap_or_else"])  // -- is ok?
        || match_def_path(cx, did, &["core", "option", "{{impl}}"])  // -- is ok?
        || match_def_path(cx, did, &["core", "ptr", "NonNull"])  // -- is ok?
        || match_def_path(cx, did, &["core", "ptr", "read"])  // -- is ok?
        || match_def_path(cx, did, &["core", "ptr", "{{impl}}"])  // -- is ok?
        || match_def_path(cx, did, &["core", "result", "Result"])  // -- is ok?
        || match_def_path(cx, did, &["core", "result", "{{impl}}", "map"])  // -- is ok?
        || match_def_path(cx, did, &["core", "result", "{{impl}}", "map_err"])  // -- is ok?
        || match_def_path(cx, did, &["core", "slice", "Iter"])
        || match_def_path(cx, did, &["core", "slice", "IterMut"])  // -- is ok?
        || match_def_path(cx, did, &["core", "slice", "{{impl}}"]) // -- TODO check if ok **autogenerated**
        || match_def_path(cx, did, &["malloc_size_of", "MallocSizeOf"])  // -- is ok?
        || match_def_path(cx, did, &["metrics", "{{impl}}", "maybe_set_tti"]) // -- TODO check if ok **autogenerated**
        || match_def_path(cx, did, &["mozjs", "conversions", "ToJSValConvertible"])  // -- is ok?
        || match_def_path(cx, did, &["ref_filter_map", "ref_filter_map"]) // -- TODO check if ok **autogenerated**
        || match_def_path(cx, did, &["script", "body", "BodyOperations"]) // -- TODO check if ok **autogenerated**
        || match_def_path(cx, did, &["script", "dom", "activation", "Activatable"]) // -- TODO check if ok **autogenerated**
        || match_def_path(cx, did, &["script", "dom", "bindings", "codegen", "Bindings", "FunctionBinding", "{{impl}}", "Call_"]) // -- TODO check if ok **autogenerated**
        || match_def_path(cx, did, &["script", "dom", "bindings", "codegen", "Bindings", "MutationObserverBinding", "{{impl}}", "Call_"]) // -- TODO check if ok **autogenerated**
        || match_def_path(cx, did, &["script", "dom", "bindings", "codegen", "Bindings", "NodeFilterBinding", "{{impl}}", "AcceptNode_"]) // -- TODO check if ok **autogenerated**
        || match_def_path(cx, did, &["script", "dom", "bindings", "codegen", "Bindings", "PromiseBinding", "{{impl}}", "Call_"]) // -- TODO check if ok **autogenerated**
        || match_def_path(cx, did, &["script", "dom", "bindings", "root", "RootedReference"]) // -- TODO check if ok **autogenerated**
        || match_def_path(cx, did, &["script", "dom", "bindings", "root", "{{impl}}", "downcast"]) // -- TODO check if ok **autogenerated**
        || match_def_path(cx, did, &["script", "dom", "bindings", "root", "{{impl}}", "upcast"]) // -- TODO check if ok **autogenerated**
        || match_def_path(cx, did, &["script", "dom", "bindings", "root", "{{impl}}"])  // -- is ok? note: probably should be treated like "new"/"new_*" method
        || match_def_path(cx, did, &["script", "dom", "bindings", "trace", "{{impl}}"]) // -- TODO check if ok **autogenerated**
        || match_def_path(cx, did, &["script", "dom", "bindings", "utils", "AsVoidPtr"]) // -- TODO check if ok **autogenerated**
        || match_def_path(cx, did, &["script", "dom", "bindings", "weakref", "MutableWeakRef"]) // -- TODO check if ok **autogenerated**
        || match_def_path(cx, did, &["script", "dom", "bindings", "weakref", "{{impl}}"]) // -- TODO check if ok **autogenerated**
        || match_def_path(cx, did, &["script", "dom", "bluetooth", "response_async"]) // -- TODO check if ok **autogenerated**
        || match_def_path(cx, did, &["script", "dom", "dompointreadonly", "DOMPointWriteMethods"]) // -- TODO check if ok **autogenerated**
        || match_def_path(cx, did, &["script", "dom", "element", "RawLayoutElementHelpers"]) // -- TODO check if ok **autogenerated**
        || match_def_path(cx, did, &["script", "dom", "element", "{{impl}}", "is_in_same_home_subtree"]) // -- TODO check if ok **autogenerated**
        || match_def_path(cx, did, &["script", "dom", "eventtarget", "{{impl}}", "call_or_handle_event"]) // -- TODO check if ok **autogenerated**
        || match_def_path(cx, did, &["script", "dom", "htmlformelement", "FormControl"]) // -- TODO check if ok **autogenerated**
        || match_def_path(cx, did, &["script", "dom", "htmlformelement", "FormControlElementHelpers"]) // -- TODO check if ok **autogenerated**
        || match_def_path(cx, did, &["script", "dom", "node", "VecPreOrderInsertionHelper"]) // -- TODO check if ok **autogenerated**
        || match_def_path(cx, did, &["script", "dom", "node", "{{impl}}", "insert_cell_or_row"]) // -- TODO check if ok **autogenerated**
        || match_def_path(cx, did, &["script", "dom", "node", "{{impl}}", "reflect_node"]) // -- TODO check if ok **autogenerated**
        || match_def_path(cx, did, &["script", "dom", "permissions", "PermissionAlgorithm"]) // -- TODO check if ok **autogenerated**
        || match_def_path(cx, did, &["script", "dom", "promise", "{{impl}}", "resolve_native"]) // -- TODO check if ok **autogenerated**
        || match_def_path(cx, did, &["script", "dom", "virtualmethods", "VirtualMethods"]) // -- TODO check if ok **autogenerated**
        || match_def_path(cx, did, &["script", "dom", "webgl_extensions", "extensions", "{{impl}}", "is_enabled"]) // -- TODO check if ok **autogenerated**
        || match_def_path(cx, did, &["script", "dom", "webgl_extensions", "extensions", "{{impl}}", "register"]) // -- TODO check if ok **autogenerated**
        || match_def_path(cx, did, &["script", "dom", "webgl_extensions", "wrapper", "WebGLExtensionWrapper"]) // -- TODO check if ok **autogenerated**
        || match_def_path(cx, did, &["script", "dom", "xmlhttprequest", "Extractable"]) // -- TODO check if ok **autogenerated**
        || match_def_path(cx, did, &["std", "collections", "hash", "map", "Entry"])
        || match_def_path(cx, did, &["std", "collections", "hash", "map", "HashMap"])  // -- is ok?
        || match_def_path(cx, did, &["std", "collections", "hash", "map", "Iter"])
        || match_def_path(cx, did, &["std", "collections", "hash", "map", "Keys"])  // -- is ok?
        || match_def_path(cx, did, &["std", "collections", "hash", "map", "OccupiedEntry"])
        || match_def_path(cx, did, &["std", "collections", "hash", "map", "VacantEntry"])
        || match_def_path(cx, did, &["std", "collections", "hash", "map", "{{impl}}"])  // -- is ok?
        || match_def_path(cx, did, &["std", "collections", "hash", "set", "HashSet"])  // -- is ok?
        || match_def_path(cx, did, &["std", "collections", "hash", "set", "Iter"])
        || match_def_path(cx, did, &["std", "collections", "hash", "set", "{{impl}}", "remove"]) // -- TODO check if ok **autogenerated**
        || match_def_path(cx, did, &["std", "collections", "hash", "set", "{{impl}}"]) // -- TODO check if ok **autogenerated**
        || match_def_path(cx, did, &["std", "sync", "mpsc", "Sender"])  // -- is ok?
        || match_def_path(cx, did, &["std", "thread", "local", "{{impl}}", "with"])  // -- is ok?
        || match_def_path(cx, did, &["style", "stylesheet_set", "DocumentStylesheetSet"])  // -- is ok?
        || match_def_path(cx, did, &["style", "stylesheet_set", "{{impl}}"]) // -- TODO check if ok **autogenerated**
    }
}

impl<'a, 'b, 'tcx> MirVisitor<'tcx> for MirFnVisitor<'a, 'b, 'tcx> {
    fn visit_local_decl(&mut self, local: mir::Local, decl: &mir::LocalDecl<'tcx>) {
        let ur_cx = &self.unrooted_cx;
        match self.mir.local_kind(local) {
            mir::LocalKind::ReturnPointer => if !self.in_derive_expn && !self.in_new_function && ur_cx.is_unrooted_ty(decl.ty, false) {
                ur_cx.late_cx.span_lint(UNROOTED_MUST_ROOT, decl.source_info.span, "Function return type must be rooted.")
            },
            mir::LocalKind::Arg => if !self.in_derive_expn && ur_cx.is_unrooted_ty(decl.ty, false) {
                ur_cx.late_cx.span_lint(UNROOTED_MUST_ROOT, decl.source_info.span, "Function argument type must be rooted.")
            },
            mir::LocalKind::Var => if ur_cx.is_unrooted_ty(decl.ty, self.in_new_function) {
                ur_cx.late_cx.span_lint(UNROOTED_MUST_ROOT, decl.source_info.span, "Type of binding/expression must be rooted.")
            },
            _ => {},
        }

        let cx = ur_cx.late_cx;
        match decl.ty.sty {
            ty::TyAdt(adt_def, substs) => {
                let mut type_info = String::new();
                if adt_def.is_box() && self.in_new_function {
                    // Boxes of unrooted types are allowed in new functions.
                }
                else if ur_cx.has_unrooted_generic_substs(adt_def.did, substs, &mut type_info) {
                    let mut msg = "ADT generic type must be rooted.\n".to_string();
                    msg.push_str(&type_info);
                    cx.span_lint(UNROOTED_MUST_ROOT, decl.source_info.span, &msg);
                }
            },
            _ => {},
        }
    }

    fn visit_constant(&mut self, constant: &mir::Constant<'tcx>, location: mir::Location) {
        self.super_constant(constant, location);

        let ur_cx = &self.unrooted_cx;
        let cx = ur_cx.late_cx;
        match constant.ty.sty {
            ty::TyFnDef(callee_def_id, callee_substs) => {
                let def_path = get_def_path(cx, callee_def_id);
                let mut type_info = String::new();
                // TODO remove following commented code
                // cx.span_lint(UNROOTED_MUST_ROOT, constant.span, &def_path.join("::")); // tmp auxiliary call
                if self.in_new_function && (// match_def_path(cx, callee_def_id, &["alloc", "boxed", "{{impl}}", "new"]) ||
                  def_path.last().unwrap().starts_with("new_") || def_path.last().unwrap() == "new") {
                      // "new"-like (constructors) functions are allowed
                }
                else if ur_cx.has_unrooted_generic_substs(callee_def_id, callee_substs, &mut type_info) {
                    // TODO tmp code - delete later
                    let mut msg = "Callee generic type must be rooted.\n".to_string();
                    //msg.push_str(&def_path.join("::"));
                    //msg.push_str(&"\n");
                    msg.push_str(&type_info);
                    cx.span_lint(UNROOTED_MUST_ROOT, constant.span, &msg);
                    //cx.span_lint(UNROOTED_MUST_ROOT, constant.span, "Callee generic type must be rooted.")
                }
            },
            _ => {},
        }
    }
}
