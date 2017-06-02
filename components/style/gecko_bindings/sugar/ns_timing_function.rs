/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use euclid::{Point2D, TypedPoint2D};
use gecko_bindings::structs::{nsTimingFunction, nsTimingFunction_Type};
use std::mem;
use values::computed::ToComputedValue;
use values::computed::transform::TimingFunction as ComputedTimingFunction;
use values::generics::transform::{StepPosition, TimingFunction as GenericTimingFunction, TimingKeyword};
use values::specified::transform::TimingFunction;

impl nsTimingFunction {
    fn set_as_step(&mut self, function_type: nsTimingFunction_Type, steps: u32) {
        debug_assert!(function_type == nsTimingFunction_Type::StepStart ||
                      function_type == nsTimingFunction_Type::StepEnd,
                      "function_type should be step-start or step-end");
        self.mType = function_type;
        unsafe {
            self.__bindgen_anon_1.__bindgen_anon_1.as_mut().mStepsOrFrames = steps;
        }
    }

    fn set_as_frames(&mut self, frames: u32) {
        self.mType = nsTimingFunction_Type::Frames;
        unsafe {
            self.__bindgen_anon_1.__bindgen_anon_1.as_mut().mStepsOrFrames = frames;
        }
    }

    fn set_as_bezier(&mut self,
                     function_type: nsTimingFunction_Type,
                     p1: Point2D<f32>,
                     p2: Point2D<f32>) {
        self.mType = function_type;
        unsafe {
            let ref mut gecko_cubic_bezier =
                unsafe { self.__bindgen_anon_1.mFunc.as_mut() };
            gecko_cubic_bezier.mX1 = p1.x;
            gecko_cubic_bezier.mY1 = p1.y;
            gecko_cubic_bezier.mX2 = p2.x;
            gecko_cubic_bezier.mY2 = p2.y;
        }
    }
}

impl From<ComputedTimingFunction> for nsTimingFunction {
    fn from(function: ComputedTimingFunction) -> nsTimingFunction {
        TimingFunction::from_computed_value(&function).into()
    }
}

impl From<TimingFunction> for nsTimingFunction {
    fn from(function: TimingFunction) -> nsTimingFunction {
        let mut tf: nsTimingFunction = unsafe { mem::zeroed() };

        match function {
            GenericTimingFunction::Steps(steps, StepPosition::Start) => {
                debug_assert!(steps.value() >= 0);
                tf.set_as_step(nsTimingFunction_Type::StepStart, steps.value() as u32);
            },
            GenericTimingFunction::Steps(steps, StepPosition::End) => {
                debug_assert!(steps.value() >= 0);
                tf.set_as_step(nsTimingFunction_Type::StepEnd, steps.value() as u32);
            },
            GenericTimingFunction::Frames(frames) => {
                debug_assert!(frames.value() >= 2);
                tf.set_as_frames(frames.value() as u32);
            },
            GenericTimingFunction::CubicBezier(p1, p2) => {
                tf.set_as_bezier(nsTimingFunction_Type::CubicBezier,
                                 Point2D::new(p1.x.get(), p1.y.get()),
                                 Point2D::new(p2.x.get(), p2.y.get()));
            },
            GenericTimingFunction::Keyword(keyword) => {
                let (p1, p2) = keyword.to_bezier_points();
                tf.set_as_bezier(keyword.into(), p1, p2)
            },
        }
        tf
    }
}

impl From<nsTimingFunction> for ComputedTimingFunction {
    fn from(function: nsTimingFunction) -> ComputedTimingFunction {
        match function.mType {
            nsTimingFunction_Type::StepStart => {
                GenericTimingFunction::Steps(
                    unsafe { function.__bindgen_anon_1.__bindgen_anon_1.as_ref().mStepsOrFrames },
                    StepPosition::Start)
            },
            nsTimingFunction_Type::StepEnd => {
                GenericTimingFunction::Steps(
                    unsafe { function.__bindgen_anon_1.__bindgen_anon_1.as_ref().mStepsOrFrames },
                    StepPosition::End)
            },
            nsTimingFunction_Type::Frames => {
                GenericTimingFunction::Frames(
                    unsafe { function.__bindgen_anon_1.__bindgen_anon_1.as_ref().mStepsOrFrames })
            }
            nsTimingFunction_Type::Ease => {
                GenericTimingFunction::Keyword(TimingKeyword::Ease)
            },
            nsTimingFunction_Type::Linear => {
                GenericTimingFunction::Keyword(TimingKeyword::Linear)
            },
            nsTimingFunction_Type::EaseIn => {
                GenericTimingFunction::Keyword(TimingKeyword::EaseIn)
            },
            nsTimingFunction_Type::EaseOut => {
                GenericTimingFunction::Keyword(TimingKeyword::EaseOut)
            },
            nsTimingFunction_Type::EaseInOut => {
                GenericTimingFunction::Keyword(TimingKeyword::EaseInOut)
            },
            nsTimingFunction_Type::CubicBezier => {
                GenericTimingFunction::CubicBezier(
                    TypedPoint2D::new(unsafe { function.__bindgen_anon_1.mFunc.as_ref().mX1 },
                                      unsafe { function.__bindgen_anon_1.mFunc.as_ref().mY1 }),
                    TypedPoint2D::new(unsafe { function.__bindgen_anon_1.mFunc.as_ref().mX2 },
                                      unsafe { function.__bindgen_anon_1.mFunc.as_ref().mY2 }))
            },
        }
    }
}

impl From<TimingKeyword> for nsTimingFunction_Type {
    fn from(keyword: TimingKeyword) -> Self {
        match keyword {
            TimingKeyword::Linear => nsTimingFunction_Type::Linear,
            TimingKeyword::Ease => nsTimingFunction_Type::Ease,
            TimingKeyword::EaseIn => nsTimingFunction_Type::EaseIn,
            TimingKeyword::EaseOut => nsTimingFunction_Type::EaseOut,
            TimingKeyword::EaseInOut => nsTimingFunction_Type::EaseInOut,
        }
    }
}
