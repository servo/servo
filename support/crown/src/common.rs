/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use rustc_hir::def::DefKind;
use rustc_hir::def_id::{CrateNum, DefId};
use rustc_infer::infer::TyCtxtInferExt;
use rustc_infer::traits::{EvaluationResult, Obligation, ObligationCause};
use rustc_lint::LateContext;
use rustc_middle::ty::{self, GenericArg, TraitRef, Ty, TyCtxt, TypeVisitableExt};
use rustc_span::hygiene::{ExpnKind, MacroKind};
use rustc_span::symbol::Symbol;
use rustc_span::{Span, DUMMY_SP};
use rustc_trait_selection::traits::query::evaluate_obligation::InferCtxtExt as _;
use rustc_type_ir::Upcast as _;

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

pub fn find_first_crate<'tcx>(tcx: &TyCtxt<'tcx>, crate_name: Symbol) -> Option<CrateNum> {
    tcx.crates(())
        .iter()
        .find(|c| tcx.crate_name(**c) == crate_name)
        .copied()
}

pub fn trait_in_crate<'tcx>(
    tcx: &TyCtxt<'tcx>,
    krate: CrateNum,
    trait_sym: Symbol,
) -> Option<DefId> {
    tcx.traits(krate)
        .iter()
        .find(|id| tcx.opt_item_name(**id) == Some(trait_sym))
        .copied()
}

/*
Stuff copied from clippy:
*/

/// Checks whether a type implements a trait.
/// The function returns false in case the type contains an inference variable.
///
/// See [Common tools for writing lints] for an example how to use this function and other options.
///
/// [Common tools for writing lints]: https://github.com/rust-lang/rust-clippy/blob/master/book/src/development/common_tools_writing_lints.md#checking-if-a-type-implements-a-specific-trait
pub fn implements_trait<'tcx>(
    cx: &LateContext<'tcx>,
    ty: Ty<'tcx>,
    trait_id: DefId,
    args: &[GenericArg<'tcx>],
) -> bool {
    implements_trait_with_env_from_iter(
        cx.tcx,
        cx.typing_env(),
        ty,
        trait_id,
        None,
        args.iter().map(|&x| Some(x)),
    )
}

/// Same as `implements_trait_from_env` but takes the arguments as an iterator.
pub fn implements_trait_with_env_from_iter<'tcx>(
    tcx: TyCtxt<'tcx>,
    typing_env: ty::TypingEnv<'tcx>,
    ty: Ty<'tcx>,
    trait_id: DefId,
    callee_id: Option<DefId>,
    args: impl IntoIterator<Item = impl Into<Option<GenericArg<'tcx>>>>,
) -> bool {
    // Clippy shouldn't have infer types
    assert!(!ty.has_infer());

    // If a `callee_id` is passed, then we assert that it is a body owner
    // through calling `body_owner_kind`, which would panic if the callee
    // does not have a body.
    if let Some(callee_id) = callee_id {
        let _ = tcx.hir_body_owner_kind(callee_id);
    }

    let ty = tcx.erase_and_anonymize_regions(ty);
    if ty.has_escaping_bound_vars() {
        return false;
    }

    let (infcx, param_env) = tcx.infer_ctxt().build_with_typing_env(typing_env);
    let args = args
        .into_iter()
        .map(|arg| {
            arg.into()
                .unwrap_or_else(|| infcx.next_ty_var(DUMMY_SP).into())
        })
        .collect::<Vec<_>>();

    let trait_ref = TraitRef::new(
        tcx,
        trait_id,
        [GenericArg::from(ty)].into_iter().chain(args),
    );

    debug_assert!(
        matches!(tcx.def_kind(trait_id), DefKind::Trait | DefKind::TraitAlias),
        "`DefId` must belong to a trait or trait alias"
    );

    let obligation = Obligation {
        cause: ObligationCause::dummy(),
        param_env,
        recursion_depth: 0,
        predicate: trait_ref.upcast(tcx),
    };
    infcx
        .evaluate_obligation(&obligation)
        .is_ok_and(EvaluationResult::must_apply_modulo_regions)
}
