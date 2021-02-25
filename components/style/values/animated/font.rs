/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Animation implementation for various font-related types.

use super::{Animate, Procedure, ToAnimatedZero};
use crate::values::computed::font::{FontVariationSettings, FontWeight};
use crate::values::computed::Number;
use crate::values::distance::{ComputeSquaredDistance, SquaredDistance};
use crate::values::generics::font::{FontSettings as GenericFontSettings, FontTag, VariationValue};

impl ToAnimatedZero for FontWeight {
    #[inline]
    fn to_animated_zero(&self) -> Result<Self, ()> {
        Ok(FontWeight::normal())
    }
}

/// <https://drafts.csswg.org/css-fonts-4/#font-variation-settings-def>
impl Animate for FontVariationSettings {
    #[inline]
    fn animate(&self, other: &Self, procedure: Procedure) -> Result<Self, ()> {
        FontSettingTagIter::new(self, other)?
            .map(|r| r.and_then(|(st, ot)| st.animate(&ot, procedure)))
            .collect::<Result<Vec<ComputedVariationValue>, ()>>()
            .map(|v| GenericFontSettings(v.into_boxed_slice()))
    }
}

impl ComputeSquaredDistance for FontVariationSettings {
    #[inline]
    fn compute_squared_distance(&self, other: &Self) -> Result<SquaredDistance, ()> {
        FontSettingTagIter::new(self, other)?
            .map(|r| r.and_then(|(st, ot)| st.compute_squared_distance(&ot)))
            .sum()
    }
}

impl ToAnimatedZero for FontVariationSettings {
    #[inline]
    fn to_animated_zero(&self) -> Result<Self, ()> {
        Err(())
    }
}

type ComputedVariationValue = VariationValue<Number>;

// FIXME: Could do a rename, this is only used for font variations.
struct FontSettingTagIterState<'a> {
    tags: Vec<&'a ComputedVariationValue>,
    index: usize,
    prev_tag: FontTag,
}

impl<'a> FontSettingTagIterState<'a> {
    fn new(tags: Vec<&'a ComputedVariationValue>) -> FontSettingTagIterState<'a> {
        FontSettingTagIterState {
            index: tags.len(),
            tags,
            prev_tag: FontTag(0),
        }
    }
}

/// Iterator for font-variation-settings tag lists
///
/// [CSS fonts level 4](https://drafts.csswg.org/css-fonts-4/#descdef-font-face-font-variation-settings)
/// defines the animation of font-variation-settings as follows:
///
///   Two declarations of font-feature-settings[sic] can be animated between if
///   they are "like".  "Like" declarations are ones where the same set of
///   properties appear (in any order).  Because succesive[sic] duplicate
///   properties are applied instead of prior duplicate properties, two
///   declarations can be "like" even if they have differing number of
///   properties. If two declarations are "like" then animation occurs pairwise
///   between corresponding values in the declarations.
///
/// In other words if we have the following lists:
///
///   "wght" 1.4, "wdth" 5, "wght" 2
///   "wdth" 8, "wght" 4, "wdth" 10
///
/// We should animate between:
///
///   "wdth" 5, "wght" 2
///   "wght" 4, "wdth" 10
///
/// This iterator supports this by sorting the two lists, then iterating them in
/// reverse, and skipping entries with repeated tag names. It will return
/// Some(Err()) if it reaches the end of one list before the other, or if the
/// tag names do not match.
///
/// For the above example, this iterator would return:
///
///   Some(Ok("wght" 2, "wght" 4))
///   Some(Ok("wdth" 5, "wdth" 10))
///   None
///
struct FontSettingTagIter<'a> {
    a_state: FontSettingTagIterState<'a>,
    b_state: FontSettingTagIterState<'a>,
}

impl<'a> FontSettingTagIter<'a> {
    fn new(
        a_settings: &'a FontVariationSettings,
        b_settings: &'a FontVariationSettings,
    ) -> Result<FontSettingTagIter<'a>, ()> {
        if a_settings.0.is_empty() || b_settings.0.is_empty() {
            return Err(());
        }

        fn as_new_sorted_tags(tags: &[ComputedVariationValue]) -> Vec<&ComputedVariationValue> {
            use std::iter::FromIterator;
            let mut sorted_tags = Vec::from_iter(tags.iter());
            sorted_tags.sort_by_key(|k| k.tag.0);
            sorted_tags
        }

        Ok(FontSettingTagIter {
            a_state: FontSettingTagIterState::new(as_new_sorted_tags(&a_settings.0)),
            b_state: FontSettingTagIterState::new(as_new_sorted_tags(&b_settings.0)),
        })
    }

    fn next_tag(state: &mut FontSettingTagIterState<'a>) -> Option<&'a ComputedVariationValue> {
        if state.index == 0 {
            return None;
        }

        state.index -= 1;
        let tag = state.tags[state.index];
        if tag.tag == state.prev_tag {
            FontSettingTagIter::next_tag(state)
        } else {
            state.prev_tag = tag.tag;
            Some(tag)
        }
    }
}

impl<'a> Iterator for FontSettingTagIter<'a> {
    type Item = Result<(&'a ComputedVariationValue, &'a ComputedVariationValue), ()>;

    fn next(
        &mut self,
    ) -> Option<Result<(&'a ComputedVariationValue, &'a ComputedVariationValue), ()>> {
        match (
            FontSettingTagIter::next_tag(&mut self.a_state),
            FontSettingTagIter::next_tag(&mut self.b_state),
        ) {
            (Some(at), Some(bt)) if at.tag == bt.tag => Some(Ok((at, bt))),
            (None, None) => None,
            _ => Some(Err(())), // Mismatch number of unique tags or tag names.
        }
    }
}
