/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use euclid::point::{Point2D, TypedPoint2D};
use gecko_bindings::structs::{nsTimingFunction, nsTimingFunction_Type};
use properties::longhands::transition_timing_function::single_value::FunctionKeyword;
use properties::longhands::transition_timing_function::single_value::SpecifiedValue as SpecifiedTimingFunction;
use properties::longhands::transition_timing_function::single_value::computed_value::StartEnd;
use properties::longhands::transition_timing_function::single_value::computed_value::T as ComputedTimingFunction;
use std::mem;

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

    fn set_as_cubic_bezier(&mut self, p1: Point2D<f32>, p2: Point2D<f32>) {
        self.mType = nsTimingFunction_Type::CubicBezier;
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
        let mut tf: nsTimingFunction = unsafe { mem::zeroed() };

        match function {
            ComputedTimingFunction::Steps(steps, StartEnd::Start) => {
                tf.set_as_step(nsTimingFunction_Type::StepStart, steps);
            },
            ComputedTimingFunction::Steps(steps, StartEnd::End) => {
                tf.set_as_step(nsTimingFunction_Type::StepEnd, steps);
            },
            ComputedTimingFunction::CubicBezier(p1, p2) => {
                tf.set_as_cubic_bezier(p1, p2);
            },
        }
        tf
    }
}

impl From<SpecifiedTimingFunction> for nsTimingFunction {
    fn from(function: SpecifiedTimingFunction) -> nsTimingFunction {
        let mut tf: nsTimingFunction = unsafe { mem::zeroed() };

        match function {
            SpecifiedTimingFunction::Steps(steps, StartEnd::Start) => {
                debug_assert!(steps.value() >= 0);
                tf.set_as_step(nsTimingFunction_Type::StepStart, steps.value() as u32);
            },
            SpecifiedTimingFunction::Steps(steps, StartEnd::End) => {
                debug_assert!(steps.value() >= 0);
                tf.set_as_step(nsTimingFunction_Type::StepEnd, steps.value() as u32);
            },
            SpecifiedTimingFunction::CubicBezier(p1, p2) => {
                tf.set_as_cubic_bezier(Point2D::new(p1.x.value, p1.y.value),
                                       Point2D::new(p2.x.value, p2.y.value));
            },
            SpecifiedTimingFunction::Keyword(keyword) => {
                match keyword {
                    FunctionKeyword::Ease => tf.mType = nsTimingFunction_Type::Ease,
                    FunctionKeyword::Linear => tf.mType = nsTimingFunction_Type::Linear,
                    FunctionKeyword::EaseIn => tf.mType = nsTimingFunction_Type::EaseIn,
                    FunctionKeyword::EaseOut => tf.mType = nsTimingFunction_Type::EaseOut,
                    FunctionKeyword::EaseInOut => tf.mType = nsTimingFunction_Type::EaseInOut,
                    FunctionKeyword::StepStart => {
                        tf.set_as_step(nsTimingFunction_Type::StepStart, 1);
                    },
                    FunctionKeyword::StepEnd => {
                        tf.set_as_step(nsTimingFunction_Type::StepEnd, 1);
                    },
                }
            },
        }
        tf
    }
}

impl From<nsTimingFunction> for ComputedTimingFunction {
    fn from(function: nsTimingFunction) -> ComputedTimingFunction {
        match function.mType {
            nsTimingFunction_Type::StepStart => {
                ComputedTimingFunction::Steps(
                    unsafe { function.__bindgen_anon_1.__bindgen_anon_1.as_ref().mStepsOrFrames },
                    StartEnd::Start)
            },
            nsTimingFunction_Type::StepEnd => {
                ComputedTimingFunction::Steps(
                    unsafe { function.__bindgen_anon_1.__bindgen_anon_1.as_ref().mStepsOrFrames },
                    StartEnd::End)
            },
            nsTimingFunction_Type::Frames => {
                // https://github.com/servo/servo/issues/15740
                panic!("Frames timing function is not support yet");
            }
            nsTimingFunction_Type::Ease |
            nsTimingFunction_Type::Linear |
            nsTimingFunction_Type::EaseIn |
            nsTimingFunction_Type::EaseOut |
            nsTimingFunction_Type::EaseInOut |
            nsTimingFunction_Type::CubicBezier => {
                ComputedTimingFunction::CubicBezier(
                    TypedPoint2D::new(unsafe { function.__bindgen_anon_1.mFunc.as_ref().mX1 },
                                      unsafe { function.__bindgen_anon_1.mFunc.as_ref().mY1 }),
                    TypedPoint2D::new(unsafe { function.__bindgen_anon_1.mFunc.as_ref().mX2 },
                                      unsafe { function.__bindgen_anon_1.mFunc.as_ref().mY2 }))
            },
        }
    }
}
