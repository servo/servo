/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Resolved values for counter properties

use super::{Context, ToResolvedValue};
use crate::values::computed;

/// https://drafts.csswg.org/css-content/#content-property
///
/// We implement this at resolved value time because otherwise it causes us to
/// allocate a bunch of useless initial structs for ::before / ::after, which is
/// a bit unfortunate.
///
/// Though these should be temporary, mostly, so if this causes complexity in
/// other places, it should be fine to move to `StyleAdjuster`.
///
/// See https://github.com/w3c/csswg-drafts/issues/4632 for where some related
/// issues are being discussed.
impl ToResolvedValue for computed::Content {
    type ResolvedValue = Self;

    #[inline]
    fn to_resolved_value(self, context: &Context) -> Self {
        let (is_pseudo, is_before_or_after, is_marker) = match context.style.pseudo() {
            Some(ref pseudo) => (true, pseudo.is_before_or_after(), pseudo.is_marker()),
            None => (false, false, false),
        };
        match self {
            Self::Normal if is_before_or_after => Self::None,
            // For now, make `content: none` compute to `normal` for pseudos
            // other than ::before, ::after and ::marker, as we don't respect it.
            // https://github.com/w3c/csswg-drafts/issues/6124
            // Ditto for non-pseudo elements if the pref is disabled.
            Self::None
                if (is_pseudo && !is_before_or_after && !is_marker) ||
                    (!is_pseudo &&
                        !static_prefs::pref!("layout.css.element-content-none.enabled")) =>
            {
                Self::Normal
            },
            other => other,
        }
    }

    #[inline]
    fn from_resolved_value(resolved: Self) -> Self {
        resolved
    }
}
