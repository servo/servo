/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use euclid::point::TypedPoint2D;
use gecko_bindings::structs::{nsTimingFunction, nsTimingFunction_Type};
use properties::longhands::transition_timing_function::single_value::computed_value::StartEnd;
use properties::longhands::transition_timing_function::single_value::computed_value::T as TransitionTimingFunction;
use std::mem;

impl From<TransitionTimingFunction> for nsTimingFunction {
    fn from(function: TransitionTimingFunction) -> nsTimingFunction {
        let mut tf: nsTimingFunction = unsafe { mem::zeroed() };

        match function {
            TransitionTimingFunction::Steps(steps, StartEnd::Start) => {
                tf.mType = nsTimingFunction_Type::StepStart;
                unsafe {
                    tf.__bindgen_anon_1.__bindgen_anon_1.as_mut().mSteps = steps;
                }
            },
            TransitionTimingFunction::Steps(steps, StartEnd::End) => {
                tf.mType = nsTimingFunction_Type::StepEnd;
                unsafe {
                    tf.__bindgen_anon_1.__bindgen_anon_1.as_mut().mSteps = steps;
                }
            },
            TransitionTimingFunction::CubicBezier(p1, p2) => {
                tf.mType = nsTimingFunction_Type::CubicBezier;
                let ref mut gecko_cubic_bezier =
                    unsafe { tf.__bindgen_anon_1.mFunc.as_mut() };
                gecko_cubic_bezier.mX1 = p1.x;
                gecko_cubic_bezier.mY1 = p1.y;
                gecko_cubic_bezier.mX2 = p2.x;
                gecko_cubic_bezier.mY2 = p2.y;
            },
            // FIXME: we need to add more types once TransitionTimingFunction
            // has more types.
        }
        tf
    }
}

impl From<nsTimingFunction> for TransitionTimingFunction {
    fn from(function: nsTimingFunction) -> TransitionTimingFunction {
        match function.mType {
            nsTimingFunction_Type::StepStart => {
                TransitionTimingFunction::Steps(unsafe { function.__bindgen_anon_1.__bindgen_anon_1.as_ref().mSteps },
                                                StartEnd::Start)
            },
            nsTimingFunction_Type::StepEnd => {
                TransitionTimingFunction::Steps(unsafe { function.__bindgen_anon_1.__bindgen_anon_1.as_ref().mSteps },
                                                StartEnd::End)
            },
            // FIXME: As above, we need to fix here.
            nsTimingFunction_Type::Ease |
            nsTimingFunction_Type::Linear |
            nsTimingFunction_Type::EaseIn |
            nsTimingFunction_Type::EaseOut |
            nsTimingFunction_Type::EaseInOut |
            nsTimingFunction_Type::CubicBezier => {
                TransitionTimingFunction::CubicBezier(
                    TypedPoint2D::new(unsafe { function.__bindgen_anon_1.mFunc.as_ref().mX1 },
                                      unsafe { function.__bindgen_anon_1.mFunc.as_ref().mY1 }),
                    TypedPoint2D::new(unsafe { function.__bindgen_anon_1.mFunc.as_ref().mX2 },
                                      unsafe { function.__bindgen_anon_1.mFunc.as_ref().mY2 }))
            },
        }
    }
}
