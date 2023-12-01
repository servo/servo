/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use rustc_ast::Mutability;
use rustc_hir::def::{DefKind, Res};
use rustc_hir::def_id::{CrateNum, DefId, LocalDefId, LOCAL_CRATE};
use rustc_hir::{ImplItemRef, ItemKind, Node, OwnerId, PrimTy, TraitItemRef};
use rustc_infer::infer::type_variable::{TypeVariableOrigin, TypeVariableOriginKind};
use rustc_infer::infer::TyCtxtInferExt;
use rustc_lint::LateContext;
use rustc_middle::ty::{self, GenericArg, ParamEnv, Ty, TyCtxt, TypeVisitableExt};
use rustc_span::source_map::{ExpnKind, MacroKind, Span};
use rustc_span::symbol::{Ident, Symbol};
use rustc_span::DUMMY_SP;
use rustc_trait_selection::infer::InferCtxtExt;
use rustc_type_ir::{FloatTy, IntTy, UintTy};

/// check if a DefId's path matches the given absolute type path
/// usage e.g. with
/// `match_def_path(cx, id, &["core", "option", "Option"])`
pub fn match_def_path(cx: &LateContext, def_id: DefId, path: &[Symbol]) -> bool {
    let def_path = cx.tcx.def_path(def_id);
    let krate = &cx.tcx.crate_name(def_path.krate);
    if krate != &path[0] {
        return false;
    }

    let path = &path[1..];
    let other = def_path.data;

    if other.len() != path.len() {
        return false;
    }

    other
        .into_iter()
        .zip(path)
        .all(|(e, p)| e.data.get_opt_name().as_ref() == Some(p))
}

pub fn in_derive_expn(span: Span) -> bool {
    matches!(
        span.ctxt().outer_expn_data().kind,
        ExpnKind::Macro(MacroKind::Derive, ..)
    )
}

#[macro_export]
macro_rules! symbols {
    ($($s: ident)+) => {
        #[derive(Clone)]
        #[allow(non_snake_case)]
        pub(crate) struct Symbols {
            $( $s: Symbol, )+
        }

        impl Symbols {
            fn new() -> Self {
                Symbols {
                    $( $s: Symbol::intern(stringify!($s)), )+
                }
            }
        }
    }
}

/*
Stuff copied from clippy:
*/

fn find_primitive_impls<'tcx>(tcx: TyCtxt<'tcx>, name: &str) -> impl Iterator<Item = DefId> + 'tcx {
    use rustc_middle::ty::fast_reject::SimplifiedType::*;
    let ty = match name {
        "bool" => BoolSimplifiedType,
        "char" => CharSimplifiedType,
        "str" => StrSimplifiedType,
        "array" => ArraySimplifiedType,
        "slice" => SliceSimplifiedType,
        // FIXME: rustdoc documents these two using just `pointer`.
        //
        // Maybe this is something we should do here too.
        "const_ptr" => PtrSimplifiedType(Mutability::Not),
        "mut_ptr" => PtrSimplifiedType(Mutability::Mut),
        "isize" => IntSimplifiedType(IntTy::Isize),
        "i8" => IntSimplifiedType(IntTy::I8),
        "i16" => IntSimplifiedType(IntTy::I16),
        "i32" => IntSimplifiedType(IntTy::I32),
        "i64" => IntSimplifiedType(IntTy::I64),
        "i128" => IntSimplifiedType(IntTy::I128),
        "usize" => UintSimplifiedType(UintTy::Usize),
        "u8" => UintSimplifiedType(UintTy::U8),
        "u16" => UintSimplifiedType(UintTy::U16),
        "u32" => UintSimplifiedType(UintTy::U32),
        "u64" => UintSimplifiedType(UintTy::U64),
        "u128" => UintSimplifiedType(UintTy::U128),
        "f32" => FloatSimplifiedType(FloatTy::F32),
        "f64" => FloatSimplifiedType(FloatTy::F64),
        _ => return [].iter().copied(),
    };

    tcx.incoherent_impls(ty).iter().copied()
}

fn non_local_item_children_by_name(tcx: TyCtxt<'_>, def_id: DefId, name: Symbol) -> Vec<Res> {
    match tcx.def_kind(def_id) {
        DefKind::Mod | DefKind::Enum | DefKind::Trait => tcx
            .module_children(def_id)
            .iter()
            .filter(|item| item.ident.name == name)
            .map(|child| child.res.expect_non_local())
            .collect(),
        DefKind::Impl { .. } => tcx
            .associated_item_def_ids(def_id)
            .iter()
            .copied()
            .filter(|assoc_def_id| tcx.item_name(*assoc_def_id) == name)
            .map(|assoc_def_id| Res::Def(tcx.def_kind(assoc_def_id), assoc_def_id))
            .collect(),
        _ => Vec::new(),
    }
}

fn local_item_children_by_name(tcx: TyCtxt<'_>, local_id: LocalDefId, name: Symbol) -> Vec<Res> {
    let hir = tcx.hir();

    let root_mod;
    let item_kind = match hir.find_by_def_id(local_id) {
        Some(Node::Crate(r#mod)) => {
            root_mod = ItemKind::Mod(r#mod);
            &root_mod
        },
        Some(Node::Item(item)) => &item.kind,
        _ => return Vec::new(),
    };

    let res = |ident: Ident, owner_id: OwnerId| {
        if ident.name == name {
            let def_id = owner_id.to_def_id();
            Some(Res::Def(tcx.def_kind(def_id), def_id))
        } else {
            None
        }
    };

    match item_kind {
        ItemKind::Mod(r#mod) => r#mod
            .item_ids
            .iter()
            .filter_map(|&item_id| res(hir.item(item_id).ident, item_id.owner_id))
            .collect(),
        ItemKind::Impl(r#impl) => r#impl
            .items
            .iter()
            .filter_map(|&ImplItemRef { ident, id, .. }| res(ident, id.owner_id))
            .collect(),
        ItemKind::Trait(.., trait_item_refs) => trait_item_refs
            .iter()
            .filter_map(|&TraitItemRef { ident, id, .. }| res(ident, id.owner_id))
            .collect(),
        _ => Vec::new(),
    }
}

fn item_children_by_name(tcx: TyCtxt<'_>, def_id: DefId, name: Symbol) -> Vec<Res> {
    if let Some(local_id) = def_id.as_local() {
        local_item_children_by_name(tcx, local_id, name)
    } else {
        non_local_item_children_by_name(tcx, def_id, name)
    }
}

/// Resolves a def path like `std::vec::Vec`.
///
/// Can return multiple resolutions when there are multiple versions of the same crate, e.g.
/// `memchr::memchr` could return the functions from both memchr 1.0 and memchr 2.0.
///
/// Also returns multiple results when there are multiple paths under the same name e.g. `std::vec`
/// would have both a [`DefKind::Mod`] and [`DefKind::Macro`].
///
/// This function is expensive and should be used sparingly.
pub fn def_path_res(cx: &LateContext<'_>, path: &[&str]) -> Vec<Res> {
    fn find_crates(tcx: TyCtxt<'_>, name: Symbol) -> impl Iterator<Item = DefId> + '_ {
        tcx.crates(())
            .iter()
            .copied()
            .filter(move |&num| tcx.crate_name(num) == name)
            .map(CrateNum::as_def_id)
    }

    let tcx = cx.tcx;

    let (base, mut path) = match *path {
        [primitive] => {
            return vec![PrimTy::from_name(Symbol::intern(primitive)).map_or(Res::Err, Res::PrimTy)];
        },
        [base, ref path @ ..] => (base, path),
        _ => return Vec::new(),
    };

    let base_sym = Symbol::intern(base);

    let local_crate = if tcx.crate_name(LOCAL_CRATE) == base_sym {
        Some(LOCAL_CRATE.as_def_id())
    } else {
        None
    };

    let starts = find_primitive_impls(tcx, base)
        .chain(find_crates(tcx, base_sym))
        .chain(local_crate)
        .map(|id| Res::Def(tcx.def_kind(id), id));

    let mut resolutions: Vec<Res> = starts.collect();

    while let [segment, rest @ ..] = path {
        path = rest;
        let segment = Symbol::intern(segment);

        resolutions = resolutions
            .into_iter()
            .filter_map(|res| res.opt_def_id())
            .flat_map(|def_id| {
                // When the current def_id is e.g. `struct S`, check the impl items in
                // `impl S { ... }`
                let inherent_impl_children = tcx
                    .inherent_impls(def_id)
                    .iter()
                    .flat_map(|&impl_def_id| item_children_by_name(tcx, impl_def_id, segment));

                let direct_children = item_children_by_name(tcx, def_id, segment);

                inherent_impl_children.chain(direct_children)
            })
            .collect();
    }

    resolutions
}

/// Resolves a def path like `std::vec::Vec`, but searches only local crate
///
/// Also returns multiple results when there are multiple paths under the same name e.g. `std::vec`
/// would have both a [`DefKind::Mod`] and [`DefKind::Macro`].
///
/// This function is less expensive than `def_path_res` and should be used sparingly.
pub fn def_local_res(cx: &LateContext<'_>, path: &str) -> Vec<Res> {
    let tcx = cx.tcx;
    let local_crate = LOCAL_CRATE.as_def_id();
    let starts = Res::Def(tcx.def_kind(local_crate), local_crate);
    let mut resolutions: Vec<Res> = vec![starts];
    let segment = Symbol::intern(path);

    resolutions = resolutions
        .into_iter()
        .filter_map(|res| res.opt_def_id())
        .flat_map(|def_id| {
            // When the current def_id is e.g. `struct S`, check the impl items in
            // `impl S { ... }`
            let inherent_impl_children = tcx
                .inherent_impls(def_id)
                .iter()
                .flat_map(|&impl_def_id| item_children_by_name(tcx, impl_def_id, segment));

            let direct_children = item_children_by_name(tcx, def_id, segment);

            inherent_impl_children.chain(direct_children)
        })
        .collect();

    resolutions
}

pub fn get_trait_def_id(cx: &LateContext<'_>, path: &[&str]) -> Option<DefId> {
    def_path_res(cx, path)
        .into_iter()
        .find_map(|res| match res {
            Res::Def(DefKind::Trait | DefKind::TraitAlias, trait_id) => Some(trait_id),
            _ => None,
        })
}

pub fn get_local_trait_def_id(cx: &LateContext<'_>, path: &str) -> Option<DefId> {
    def_local_res(cx, path)
        .into_iter()
        .find_map(|res| match res {
            Res::Def(DefKind::Trait | DefKind::TraitAlias, trait_id) => Some(trait_id),
            _ => None,
        })
}

/// Checks whether a type implements a trait.
/// The function returns false in case the type contains an inference variable.
///
/// See:
/// * [`get_trait_def_id`](super::get_trait_def_id) to get a trait [`DefId`].
/// * [Common tools for writing lints] for an example how to use this function and other options.
///
/// [Common tools for writing lints]: https://github.com/rust-lang/rust-clippy/blob/master/book/src/development/common_tools_writing_lints.md#checking-if-a-type-implements-a-specific-trait
pub fn implements_trait<'tcx>(
    cx: &LateContext<'tcx>,
    ty: Ty<'tcx>,
    trait_id: DefId,
    ty_params: &[GenericArg<'tcx>],
) -> bool {
    implements_trait_with_env(
        cx.tcx,
        cx.param_env,
        ty,
        trait_id,
        ty_params.iter().map(|&arg| Some(arg)),
    )
}

/// Same as `implements_trait` but allows using a `ParamEnv` different from the lint context.
pub fn implements_trait_with_env<'tcx>(
    tcx: TyCtxt<'tcx>,
    param_env: ParamEnv<'tcx>,
    ty: ty::Ty<'tcx>,
    trait_id: DefId,
    ty_params: impl IntoIterator<Item = Option<GenericArg<'tcx>>>,
) -> bool {
    let ty = tcx.erase_regions(ty);
    if ty.has_escaping_bound_vars() {
        return false;
    }
    let infcx = tcx.infer_ctxt().build();
    let orig = TypeVariableOrigin {
        kind: TypeVariableOriginKind::MiscVariable,
        span: DUMMY_SP,
    };
    let ty_params = tcx.mk_substs_from_iter(
        ty_params
            .into_iter()
            .map(|arg| arg.unwrap_or_else(|| infcx.next_ty_var(orig).into())),
    );
    infcx
        .type_implements_trait(
            trait_id,
            // for some unknown reason we need to have vec here
            // clippy has array
            vec![ty.into()].into_iter().chain(ty_params),
            param_env,
        )
        .must_apply_modulo_regions()
}
