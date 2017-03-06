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
        let mut tf: nsTimingFunction = unsafe { mem::zeroed() };

        match function {
            ComputedTimingFunction::Steps(steps, StartEnd::Start) => {
                tf.set_as_step(nsTimingFunction_Type::StepStart, steps);
            },
            ComputedTimingFunction::Steps(steps, StartEnd::End) => {
                tf.set_as_step(nsTimingFunction_Type::StepEnd, steps);
            },
            ComputedTimingFunction::Frames(frames) => {
                tf.set_as_frames(frames);
            },
            ComputedTimingFunction::CubicBezier(p1, p2) => {
                tf.set_as_bezier(nsTimingFunction_Type::CubicBezier, p1, p2);
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
            SpecifiedTimingFunction::Frames(frames) => {
                debug_assert!(frames.value() >= 2);
                tf.set_as_frames(frames.value() as u32);
            },
            SpecifiedTimingFunction::CubicBezier(p1, p2) => {
                tf.set_as_bezier(nsTimingFunction_Type::CubicBezier,
                                 Point2D::new(p1.x.get(), p1.y.get()),
                                 Point2D::new(p2.x.get(), p2.y.get()));
            },
            SpecifiedTimingFunction::Keyword(keyword) => {
                match keyword.to_computed_value() {
                    ComputedTimingFunction::CubicBezier(p1, p2) => {
                        match keyword {
                            FunctionKeyword::Ease => {
                                tf.set_as_bezier(nsTimingFunction_Type::Ease, p1, p2);
                            },
                            FunctionKeyword::Linear => {
                                tf.set_as_bezier(nsTimingFunction_Type::Linear, p1, p2);
                            },
                            FunctionKeyword::EaseIn => {
                                tf.set_as_bezier(nsTimingFunction_Type::EaseIn, p1, p2);
                            },
                            FunctionKeyword::EaseOut => {
                                tf.set_as_bezier(nsTimingFunction_Type::EaseOut, p1, p2);
                            },
                            FunctionKeyword::EaseInOut => {
                                tf.set_as_bezier(nsTimingFunction_Type::EaseInOut, p1, p2);
                            },
                            _ => unreachable!("Unexpected bezier function type"),
                        }
                    },
                    ComputedTimingFunction::Steps(steps, StartEnd::Start) => {
                        debug_assert!(keyword == FunctionKeyword::StepStart && steps == 1);
                        tf.set_as_step(nsTimingFunction_Type::StepStart, steps);
                    },
                    ComputedTimingFunction::Steps(steps, StartEnd::End) => {
                        debug_assert!(keyword == FunctionKeyword::StepEnd && steps == 1);
                        tf.set_as_step(nsTimingFunction_Type::StepEnd, steps);
                    },
                    ComputedTimingFunction::Frames(frames) => {
                        tf.set_as_frames(frames)
                    }
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
                ComputedTimingFunction::Frames(
                    unsafe { function.__bindgen_anon_1.__bindgen_anon_1.as_ref().mStepsOrFrames })
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
