/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Animation implementation for various grid-related types.

// Note: we can implement Animate on their generic types directly, but in this case we need to
// make sure two trait bounds, L: Clone and I: PartialEq, are satisfied on almost all the
// grid-related types and their other trait implementations because Animate needs them. So in
// order to avoid adding these two trait bounds (or maybe more..) everywhere, we implement
// Animate for the computed types, instead of the generic types.

use super::{Animate, Procedure, ToAnimatedZero};
use crate::values::computed::Integer;
use crate::values::computed::LengthPercentage;
use crate::values::computed::{GridTemplateComponent, TrackList, TrackSize};
use crate::values::distance::{ComputeSquaredDistance, SquaredDistance};
use crate::values::generics::grid as generics;

fn discrete<T: Clone>(from: &T, to: &T, procedure: Procedure) -> Result<T, ()> {
    if let Procedure::Interpolate { progress } = procedure {
        Ok(if progress < 0.5 {
            from.clone()
        } else {
            to.clone()
        })
    } else {
        Err(())
    }
}

fn animate_with_discrete_fallback<T: Animate + Clone>(
    from: &T,
    to: &T,
    procedure: Procedure,
) -> Result<T, ()> {
    from.animate(to, procedure)
        .or_else(|_| discrete(from, to, procedure))
}

impl Animate for TrackSize {
    fn animate(&self, other: &Self, procedure: Procedure) -> Result<Self, ()> {
        match (self, other) {
            (&generics::TrackSize::Breadth(ref from), &generics::TrackSize::Breadth(ref to)) => {
                animate_with_discrete_fallback(from, to, procedure)
                    .map(generics::TrackSize::Breadth)
            },
            (
                &generics::TrackSize::Minmax(ref from_min, ref from_max),
                &generics::TrackSize::Minmax(ref to_min, ref to_max),
            ) => Ok(generics::TrackSize::Minmax(
                animate_with_discrete_fallback(from_min, to_min, procedure)?,
                animate_with_discrete_fallback(from_max, to_max, procedure)?,
            )),
            (
                &generics::TrackSize::FitContent(ref from),
                &generics::TrackSize::FitContent(ref to),
            ) => animate_with_discrete_fallback(from, to, procedure)
                .map(generics::TrackSize::FitContent),
            (_, _) => discrete(self, other, procedure),
        }
    }
}

impl Animate for generics::TrackRepeat<LengthPercentage, Integer> {
    fn animate(&self, other: &Self, procedure: Procedure) -> Result<Self, ()> {
        // If the keyword, auto-fit/fill, is the same it can result in different
        // number of tracks. For both auto-fit/fill, the number of columns isn't
        // known until you do layout since it depends on the container size, item
        // placement and other factors, so we cannot do the correct interpolation
        // by computed values. Therefore, return Err(()) if it's keywords. If it
        // is Number, we support animation only if the count is the same and the
        // length of track_sizes is the same.
        // https://github.com/w3c/csswg-drafts/issues/3503
        match (&self.count, &other.count) {
            (&generics::RepeatCount::Number(from), &generics::RepeatCount::Number(to))
                if from == to =>
            {
                ()
            },
            (_, _) => return Err(()),
        }

        // The length of track_sizes should be matched.
        if self.track_sizes.len() != other.track_sizes.len() {
            return Err(());
        }

        let count = self.count;
        let track_sizes = self
            .track_sizes
            .iter()
            .zip(other.track_sizes.iter())
            .map(|(a, b)| a.animate(b, procedure))
            .collect::<Result<Vec<_>, _>>()?;

        // The length of |line_names| is always 0 or N+1, where N is the length
        // of |track_sizes|. Besides, <line-names> is always discrete.
        let line_names = discrete(&self.line_names, &other.line_names, procedure)?;

        Ok(generics::TrackRepeat {
            count,
            line_names,
            track_sizes: track_sizes.into(),
        })
    }
}

impl Animate for TrackList {
    // Based on https://github.com/w3c/csswg-drafts/issues/3201:
    // 1. Check interpolation type per track, so we need to handle discrete animations
    //    in TrackSize, so any Err(()) returned from TrackSize doesn't make all TrackSize
    //    fallback to discrete animation.
    // 2. line-names is always discrete.
    fn animate(&self, other: &Self, procedure: Procedure) -> Result<Self, ()> {
        if self.values.len() != other.values.len() {
            return Err(());
        }

        if self.is_explicit() != other.is_explicit() {
            return Err(());
        }

        // For now, repeat(auto-fill/auto-fit, ...) is not animatable.
        // TrackRepeat will return Err(()) if we use keywords. Therefore, we can
        // early return here to avoid traversing |values| in <auto-track-list>.
        // This may be updated in the future.
        // https://github.com/w3c/csswg-drafts/issues/3503
        if self.has_auto_repeat() || other.has_auto_repeat() {
            return Err(());
        }

        let values = self
            .values
            .iter()
            .zip(other.values.iter())
            .map(|(a, b)| a.animate(b, procedure))
            .collect::<Result<Vec<_>, _>>()?;
        // The length of |line_names| is always 0 or N+1, where N is the length
        // of |track_sizes|. Besides, <line-names> is always discrete.
        let line_names = discrete(&self.line_names, &other.line_names, procedure)?;

        Ok(TrackList {
            values: values.into(),
            line_names,
            auto_repeat_index: self.auto_repeat_index,
        })
    }
}

impl ComputeSquaredDistance for GridTemplateComponent {
    #[inline]
    fn compute_squared_distance(&self, _other: &Self) -> Result<SquaredDistance, ()> {
        // TODO: Bug 1518585, we should implement ComputeSquaredDistance.
        Err(())
    }
}

impl ToAnimatedZero for GridTemplateComponent {
    #[inline]
    fn to_animated_zero(&self) -> Result<Self, ()> {
        // It's not clear to get a zero grid track list based on the current definition
        // of spec, so we return Err(()) directly.
        Err(())
    }
}
