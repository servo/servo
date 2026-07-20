/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use cssparser::{Parser, ParserInput};
use dom_struct::dom_struct;
use script_bindings::cell::DomRefCell;
use script_bindings::codegen::GenericBindings::AnimationEffectBinding::{
    AnimationEffectMethods, ComputedEffectTiming, EffectTiming, FillMode, OptionalEffectTiming,
    PlaybackDirection,
};
use script_bindings::codegen::GenericBindings::WindowBinding::WindowMethods;
use script_bindings::error::{Error, Fallible};
use script_bindings::num::Finite;
use script_bindings::reflector::Reflector;
use script_bindings::root::Dom;
use style::parser::Parse;
use style::stylesheets::CssRuleType;
use style::values::generics::easing::TimingKeyword;
use style::values::specified::TimingFunction;
use style_traits::{ParsingMode, ToCss};

use crate::css::parser_context_for_document;
use crate::dom::Window;
use crate::dom::bindings::codegen::UnionTypes::UnrestrictedDoubleOrString;

/// <https://drafts.csswg.org/web-animations-1/#animationeffect>
#[dom_struct]
pub(crate) struct AnimationEffect {
    reflector: Reflector,

    /// The window that this `AnimationEffect` was constructed in.
    window: Dom<Window>,

    specified_timing_properties: DomRefCell<SpecifiedTimingProperties>,
}

#[derive(Clone, JSTraceable, MallocSizeOf)]
struct SpecifiedTimingProperties {
    /// <https://drafts.csswg.org/web-animations-1/#start-delay>
    start_delay: Finite<f64>,

    /// <https://drafts.csswg.org/web-animations-1/#end-delay>
    end_delay: Finite<f64>,

    /// <https://drafts.csswg.org/web-animations-1/#fill-mode>
    fill_mode: FillMode,

    /// <https://drafts.csswg.org/web-animations-1/#iteration-count>
    iteration_count: f64,

    /// <https://drafts.csswg.org/web-animations-1/#iteration-start>
    iteration_start: Finite<f64>,

    /// <https://drafts.csswg.org/web-animations-1/#iteration-duration>
    iteration_duration: IterationDurationOrAuto,

    /// <https://drafts.csswg.org/web-animations-1/#playback-direction>
    playback_direction: PlaybackDirection,

    /// <https://drafts.csswg.org/css-easing-2/#easing-function>
    #[no_trace]
    easing_function: TimingFunction,
}

impl AnimationEffect {
    pub(crate) fn new_inherited(window: &Window) -> Self {
        Self {
            reflector: Reflector::new(),
            window: Dom::from_ref(window),

            // The default values of the timing properties specified here don't matter.
            // There is no way to construct a AnimationEffect without subsequently initializing them,
            // even if they're not passed to the constructor.
            specified_timing_properties: DomRefCell::new(SpecifiedTimingProperties {
                start_delay: Default::default(),
                end_delay: Default::default(),
                fill_mode: FillMode::None,
                iteration_count: Default::default(),
                iteration_start: Default::default(),
                iteration_duration: IterationDurationOrAuto::Auto,
                playback_direction: PlaybackDirection::Normal,
                easing_function: TimingFunction::Keyword(TimingKeyword::Linear),
            }),
        }
    }

    pub(crate) fn window(&self) -> &Window {
        &self.window
    }

    /// <https://drafts.csswg.org/web-animations-1/#update-the-timing-properties-of-an-animation-effect>
    pub(crate) fn update_the_timing_properties(
        &self,
        input: &OptionalEffectTiming,
    ) -> Fallible<()> {
        // Step 1. If the iterationStart member of input exists and is less than zero,
        // throw a TypeError and abort this procedure.
        if input
            .iterationStart
            .is_some_and(|iteration_start| *iteration_start < 0.0)
        {
            return Err(Error::Type(
                c"Negative values for iterationStart are not allowed".to_owned(),
            ));
        }

        // Step 2. If the iterations member of input exists, and is less than zero or is the value NaN,
        // throw a TypeError and abort this procedure.
        if input
            .iterations
            .is_some_and(|iterations| iterations < 0.0 || iterations.is_nan())
        {
            return Err(Error::Type(
                c"\"iterations\" must be a positive number".to_owned(),
            ));
        }

        // Step 3. If the duration member of input exists, and is less than zero or is the value NaN,
        // throw a TypeError and abort this procedure.
        //
        // Note: "auto" values are treated as zero: https://drafts.csswg.org/web-animations-1/#dom-animationeffect-updatetiming
        // > In this level of this specification, the string value auto is treated as the value zero
        // > for the purpose of timing model calculations and for the result of the duration member returned
        // > from getComputedTiming(). If the author specifies the auto value, user agents must, however,
        // > return auto for the duration member returned from getTiming().
        //
        // Note: It is unspecified how non-"auto" strings should be treated. We choose to throw a TypeError.
        //       See also https://github.com/w3c/csswg-drafts/issues/14206
        let Ok(duration) = input
            .duration
            .as_ref()
            .map(|duration| match duration {
                UnrestrictedDoubleOrString::UnrestrictedDouble(double) => {
                    if *double < 0.0 || double.is_nan() {
                        Err(())
                    } else {
                        Ok(IterationDurationOrAuto::Duration(*double))
                    }
                },
                UnrestrictedDoubleOrString::String(string) => {
                    if string == "auto" {
                        Ok(IterationDurationOrAuto::Auto)
                    } else {
                        Err(())
                    }
                },
            })
            .transpose()
        else {
            return Err(Error::Type(
                c"\"duration\" must be a positive number".to_owned(),
            ));
        };

        // Step 4. If the easing member of input exists but cannot be parsed using the <easing-function> production [CSS-EASING-1],
        // throw a TypeError and abort this procedure.
        let easing = input.easing.as_ref().and_then(|easing| {
            let easing = easing.str();
            let mut parser_input = ParserInput::new(&easing);
            let mut parser = Parser::new(&mut parser_input);

            // None of these values should matter
            let document = self.window.Document();
            let urlextradata = document.url().into_url().into();
            let parser_context = parser_context_for_document(
                &document,
                CssRuleType::Style,
                ParsingMode::DEFAULT,
                &urlextradata,
            );
            TimingFunction::parse(&parser_context, &mut parser).ok()
        });

        // Step 5. Assign each member that exists in input to the corresponding timing property of effect as follows:
        // delay → start delay
        let mut specified_timing_properties = self.specified_timing_properties.borrow_mut();
        if let Some(start_delay) = input.delay {
            specified_timing_properties.start_delay = start_delay;
        }

        // endDelay → end delay
        if let Some(end_delay) = input.endDelay {
            specified_timing_properties.end_delay = end_delay;
        }

        // fill → fill mode
        if let Some(fill) = input.fill {
            specified_timing_properties.fill_mode = fill;
        }

        // iterationStart → iteration start
        if let Some(iteration_start) = input.iterationStart {
            specified_timing_properties.iteration_start = iteration_start;
        }

        // iterations → iteration count
        if let Some(iterations) = input.iterations {
            specified_timing_properties.iteration_count = iterations;
        }

        // duration → iteration duration
        if let Some(duration) = duration {
            specified_timing_properties.iteration_duration = duration;
        }

        // direction → playback direction
        if let Some(direction) = input.direction {
            specified_timing_properties.playback_direction = direction;
        }

        // easing → easing function
        if let Some(easing) = easing {
            specified_timing_properties.easing_function = easing;
        }
        Ok(())
    }
}

impl AnimationEffectMethods<crate::DomTypeHolder> for AnimationEffect {
    /// <https://drafts.csswg.org/web-animations-1/#dom-animationeffect-gettiming>
    fn GetTiming(&self) -> EffectTiming {
        // > Returns the specified timing properties for this animation effect.
        let specified_timing_properties = self.specified_timing_properties.borrow();
        EffectTiming {
            delay: specified_timing_properties.start_delay,
            direction: specified_timing_properties.playback_direction,
            duration: specified_timing_properties.iteration_duration.into(),
            easing: specified_timing_properties
                .easing_function
                .to_css_string()
                .into(),
            endDelay: specified_timing_properties.end_delay,
            fill: specified_timing_properties.fill_mode,
            iterationStart: specified_timing_properties.iteration_start,
            iterations: specified_timing_properties.iteration_count,
        }
    }

    /// <https://drafts.csswg.org/web-animations-1/#dom-animationeffect-getcomputedtiming>
    fn GetComputedTiming(&self) -> ComputedEffectTiming {
        let specified_timing_properties = self.specified_timing_properties.borrow();
        let computed_fill_mode = if specified_timing_properties.fill_mode == FillMode::Auto {
            FillMode::None
        } else {
            specified_timing_properties.fill_mode
        };
        ComputedEffectTiming {
            delay: specified_timing_properties.start_delay,
            direction: specified_timing_properties.playback_direction,
            duration: specified_timing_properties
                .iteration_duration
                .computed_value(),
            easing: specified_timing_properties
                .easing_function
                .to_css_string()
                .into(),
            endDelay: specified_timing_properties.end_delay,
            fill: computed_fill_mode,
            iterationStart: specified_timing_properties.iteration_start,
            iterations: specified_timing_properties.iteration_count,
        }
    }

    /// <https://drafts.csswg.org/web-animations-1/#dom-animationeffect-updatetiming>
    fn UpdateTiming(&self, timing: &OptionalEffectTiming) -> Fallible<()> {
        // > Updates the specified timing properties of this animation effect by performing the procedure
        // > to update the timing properties of an animation effect passing the timing parameter as input.
        self.update_the_timing_properties(timing)
    }
}

#[derive(Clone, Copy, Debug, JSTraceable, MallocSizeOf)]
enum IterationDurationOrAuto {
    Duration(f64),
    Auto,
}

impl IterationDurationOrAuto {
    fn computed_value(&self) -> f64 {
        match self {
            IterationDurationOrAuto::Auto => 0.0,
            IterationDurationOrAuto::Duration(double) => double,
        }
    }
}

impl From<IterationDurationOrAuto> for UnrestrictedDoubleOrString {
    fn from(value: IterationDurationOrAuto) -> Self {
        match value {
            IterationDurationOrAuto::Auto => UnrestrictedDoubleOrString::String("auto".into()),
            IterationDurationOrAuto::Duration(double) => {
                UnrestrictedDoubleOrString::UnrestrictedDouble(double)
            },
        }
    }
}
