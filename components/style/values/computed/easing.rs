/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Computed types for CSS Easing functions.

use crate::bezier::Bezier;
use crate::piecewise_linear::{PiecewiseLinearFunctionBuildParameters, PiecewiseLinearFunction};
use crate::values::computed::{Integer, Number, Percentage};
use crate::values::generics::easing::{self, StepPosition, TimingKeyword};

/// A computed timing function.
pub type ComputedTimingFunction = easing::TimingFunction<Integer, Number, Percentage>;

/// An alias of the computed timing function.
pub type TimingFunction = ComputedTimingFunction;

/// A computed linear easing entry.
pub type ComputedLinearStop = easing::LinearStop<Number, Percentage>;

impl ComputedLinearStop {
    /// Convert this type to entries that can be used to build PiecewiseLinearFunction.
    pub fn to_piecewise_linear_build_parameters(
        x: &ComputedLinearStop,
    ) -> PiecewiseLinearFunctionBuildParameters {
        (
            x.output,
            x.input.into_rust().map(|x| x.0),
        )
    }
}

impl ComputedTimingFunction {
    fn calculate_step_output(steps: i32, pos: StepPosition, progress: f64) -> f64 {
        let mut current_step = (progress * (steps as f64)).floor() as i32;

        if pos == StepPosition::Start ||
            pos == StepPosition::JumpStart ||
            pos == StepPosition::JumpBoth
        {
            current_step = current_step + 1;
        }

        // FIXME: We should update current_step according to the "before flag".
        // In order to get the before flag, we have to know the current animation phase
        // and whether the iteration is reversed. For now, we skip this calculation.
        // (i.e. Treat before_flag is unset,)
        // https://drafts.csswg.org/css-easing/#step-timing-function-algo

        if progress >= 0.0 && current_step < 0 {
            current_step = 0;
        }

        let jumps = match pos {
            StepPosition::JumpBoth => steps + 1,
            StepPosition::JumpNone => steps - 1,
            StepPosition::JumpStart |
            StepPosition::JumpEnd |
            StepPosition::Start |
            StepPosition::End => steps,
        };

        if progress <= 1.0 && current_step > jumps {
            current_step = jumps;
        }

        (current_step as f64) / (jumps as f64)
    }

    /// The output of the timing function given the progress ratio of this animation.
    pub fn calculate_output(&self, progress: f64, epsilon: f64) -> f64 {
        match self {
            TimingFunction::CubicBezier { x1, y1, x2, y2 } => {
                Bezier::new(*x1, *y1, *x2, *y2).solve(progress, epsilon)
            },
            TimingFunction::Steps(steps, pos) => {
                Self::calculate_step_output(*steps, *pos, progress)
            },
            TimingFunction::LinearFunction(elements) => {
                // TODO(dshin): For servo, which uses this code path, constructing the function
                // every time the animation advances seem... expensive.
                PiecewiseLinearFunction::from_iter(
                    elements
                        .iter()
                        .map(ComputedLinearStop::to_piecewise_linear_build_parameters),
                )
                .at(progress as f32)
                .into()
            },
            TimingFunction::Keyword(keyword) => {
                let bezier = match keyword {
                    TimingKeyword::Linear => return progress,
                    TimingKeyword::Ease => Bezier::new(0.25, 0.1, 0.25, 1.),
                    TimingKeyword::EaseIn => Bezier::new(0.42, 0., 1., 1.),
                    TimingKeyword::EaseOut => Bezier::new(0., 0., 0.58, 1.),
                    TimingKeyword::EaseInOut => Bezier::new(0.42, 0., 0.58, 1.),
                };
                bezier.solve(progress, epsilon)
            },
        }
    }
}
