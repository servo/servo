/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

<%namespace name="helpers" file="/helpers.mako.rs" />

<%helpers:shorthand name="overflow" sub_properties="overflow-x overflow-y"
                    spec="https://drafts.csswg.org/css-overflow/#propdef-overflow">
    use properties::longhands::{overflow_x, overflow_y};

    pub fn parse_value(context: &ParserContext, input: &mut Parser) -> Result<Longhands, ()> {
        let overflow = try!(overflow_x::parse(context, input));
        Ok(Longhands {
            overflow_x: overflow,
            overflow_y: overflow_y::SpecifiedValue(overflow),
        })
    }

    impl<'a> ToCss for LonghandsToSerialize<'a>  {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            if *self.overflow_x == self.overflow_y.0 {
                self.overflow_x.to_css(dest)
            } else {
                Ok(())
            }
        }
    }
</%helpers:shorthand>

macro_rules! try_parse_one {
    ($input: expr, $var: ident, $prop_module: ident) => {
        if $var.is_none() {
            if let Ok(value) = $input.try($prop_module::SingleSpecifiedValue::parse) {
                $var = Some(value);
                continue;
            }
        }
    };
    ($context: expr, $input: expr, $var: ident, $prop_module: ident) => {
        if $var.is_none() {
            if let Ok(value) = $input.try(|i| {
                $prop_module::SingleSpecifiedValue::parse($context, i)
            }) {
                $var = Some(value);
                continue;
            }
        }
    };
}

<%helpers:shorthand name="transition" extra_prefixes="moz webkit"
                    sub_properties="transition-property transition-duration
                                    transition-timing-function
                                    transition-delay"
                    spec="https://drafts.csswg.org/css-transitions/#propdef-transition">
    use parser::Parse;
    use properties::longhands::{transition_delay, transition_duration, transition_property};
    use properties::longhands::{transition_timing_function};

    pub fn parse_value(context: &ParserContext, input: &mut Parser) -> Result<Longhands, ()> {
        struct SingleTransition {
            transition_property: transition_property::SingleSpecifiedValue,
            transition_duration: transition_duration::SingleSpecifiedValue,
            transition_timing_function: transition_timing_function::SingleSpecifiedValue,
            transition_delay: transition_delay::SingleSpecifiedValue,
        }

        fn parse_one_transition(context: &ParserContext, input: &mut Parser) -> Result<SingleTransition,()> {
            let (mut property, mut duration) = (None, None);
            let (mut timing_function, mut delay) = (None, None);
            loop {
                try_parse_one!(input, property, transition_property);
                try_parse_one!(context, input, duration, transition_duration);
                try_parse_one!(context, input, timing_function, transition_timing_function);
                try_parse_one!(context, input, delay, transition_delay);

                break
            }

            if let Some(property) = property {
                Ok(SingleTransition {
                    transition_property: property,
                    transition_duration:
                        duration.unwrap_or_else(transition_duration::single_value::get_initial_value),
                    transition_timing_function:
                        timing_function.unwrap_or_else(transition_timing_function::single_value
                                                                                 ::get_initial_specified_value),
                    transition_delay:
                        delay.unwrap_or_else(transition_delay::single_value::get_initial_value),
                })
            } else {
                Err(())
            }
        }

        let (mut properties, mut durations) = (Vec::new(), Vec::new());
        let (mut timing_functions, mut delays) = (Vec::new(), Vec::new());

        if input.try(|input| input.expect_ident_matching("none")).is_err() {
            let results = try!(input.parse_comma_separated(|i| parse_one_transition(context, i)));
            for result in results {
                properties.push(result.transition_property);
                durations.push(result.transition_duration);
                timing_functions.push(result.transition_timing_function);
                delays.push(result.transition_delay);
            }
        }

        Ok(Longhands {
            transition_property: transition_property::SpecifiedValue(properties),
            transition_duration: transition_duration::SpecifiedValue(durations),
            transition_timing_function:
                transition_timing_function::SpecifiedValue(timing_functions),
            transition_delay: transition_delay::SpecifiedValue(delays),
        })
    }

    impl<'a> ToCss for LonghandsToSerialize<'a>  {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            let len = self.transition_property.0.len();
            // There should be at least one declared value
            if len == 0 {
                return Ok(());
            }

            // If any value list length is differs then we don't do a shorthand serialization
            // either.
            % for name in "property duration delay timing_function".split():
                if len != self.transition_${name}.0.len() {
                    return Ok(());
                }
            % endfor

            for i in 0..len {
                if i != 0 {
                    write!(dest, ", ")?;
                }
                self.transition_property.0[i].to_css(dest)?;
                % for name in "duration timing_function delay".split():
                    dest.write_str(" ")?;
                    self.transition_${name}.0[i].to_css(dest)?;
                % endfor
            }
            Ok(())
        }
    }
</%helpers:shorthand>

<%helpers:shorthand name="animation" extra_prefixes="moz webkit"
                    sub_properties="animation-name animation-duration
                                    animation-timing-function animation-delay
                                    animation-iteration-count animation-direction
                                    animation-fill-mode animation-play-state"
                    allowed_in_keyframe_block="False"
                    spec="https://drafts.csswg.org/css-animations/#propdef-animation">
    use parser::Parse;
    use properties::longhands::{animation_name, animation_duration, animation_timing_function};
    use properties::longhands::{animation_delay, animation_iteration_count, animation_direction};
    use properties::longhands::{animation_fill_mode, animation_play_state};

    pub fn parse_value(context: &ParserContext, input: &mut Parser) -> Result<Longhands, ()> {
        struct SingleAnimation {
            animation_name: animation_name::SingleSpecifiedValue,
            animation_duration: animation_duration::SingleSpecifiedValue,
            animation_timing_function: animation_timing_function::SingleSpecifiedValue,
            animation_delay: animation_delay::SingleSpecifiedValue,
            animation_iteration_count: animation_iteration_count::SingleSpecifiedValue,
            animation_direction: animation_direction::SingleSpecifiedValue,
            animation_fill_mode: animation_fill_mode::SingleSpecifiedValue,
            animation_play_state: animation_play_state::SingleSpecifiedValue,
        }

        fn parse_one_animation(context: &ParserContext, input: &mut Parser) -> Result<SingleAnimation,()> {
            let mut duration = None;
            let mut timing_function = None;
            let mut delay = None;
            let mut iteration_count = None;
            let mut direction = None;
            let mut fill_mode = None;
            let mut play_state = None;
            let mut name = None;

            // NB: Name must be the last one here so that keywords valid for other
            // longhands are not interpreted as names.
            //
            // Also, duration must be before delay, see
            // https://drafts.csswg.org/css-animations/#typedef-single-animation
            loop {
                try_parse_one!(context, input, duration, animation_duration);
                try_parse_one!(context, input, timing_function, animation_timing_function);
                try_parse_one!(context, input, delay, animation_delay);
                try_parse_one!(context, input, iteration_count, animation_iteration_count);
                try_parse_one!(input, direction, animation_direction);
                try_parse_one!(input, fill_mode, animation_fill_mode);
                try_parse_one!(input, play_state, animation_play_state);
                try_parse_one!(context, input, name, animation_name);

                break
            }

            if let Some(name) = name {
                Ok(SingleAnimation {
                    animation_name: name,
                    animation_duration:
                        duration.unwrap_or_else(animation_duration::single_value::get_initial_value),
                    animation_timing_function:
                        timing_function.unwrap_or_else(animation_timing_function::single_value
                                                                                ::get_initial_specified_value),
                    animation_delay:
                        delay.unwrap_or_else(animation_delay::single_value::get_initial_value),
                    animation_iteration_count:
                        iteration_count.unwrap_or_else(animation_iteration_count::single_value::get_initial_value),
                    animation_direction:
                        direction.unwrap_or_else(animation_direction::single_value::get_initial_value),
                    animation_fill_mode:
                        fill_mode.unwrap_or_else(animation_fill_mode::single_value::get_initial_value),
                    animation_play_state:
                        play_state.unwrap_or_else(animation_play_state::single_value::get_initial_value),
                })
            } else {
                Err(())
            }
        }

        let mut names = vec![];
        let mut durations = vec![];
        let mut timing_functions = vec![];
        let mut delays = vec![];
        let mut iteration_counts = vec![];
        let mut directions = vec![];
        let mut fill_modes = vec![];
        let mut play_states = vec![];

        if input.try(|input| input.expect_ident_matching("none")).is_err() {
            let results = try!(input.parse_comma_separated(|i| parse_one_animation(context, i)));
            for result in results.into_iter() {
                names.push(result.animation_name);
                durations.push(result.animation_duration);
                timing_functions.push(result.animation_timing_function);
                delays.push(result.animation_delay);
                iteration_counts.push(result.animation_iteration_count);
                directions.push(result.animation_direction);
                fill_modes.push(result.animation_fill_mode);
                play_states.push(result.animation_play_state);
            }
        }

        Ok(Longhands {
            animation_name: animation_name::SpecifiedValue(names),
            animation_duration: animation_duration::SpecifiedValue(durations),
            animation_timing_function: animation_timing_function::SpecifiedValue(timing_functions),
            animation_delay: animation_delay::SpecifiedValue(delays),
            animation_iteration_count: animation_iteration_count::SpecifiedValue(iteration_counts),
            animation_direction: animation_direction::SpecifiedValue(directions),
            animation_fill_mode: animation_fill_mode::SpecifiedValue(fill_modes),
            animation_play_state: animation_play_state::SpecifiedValue(play_states),
        })
    }

    impl<'a> ToCss for LonghandsToSerialize<'a>  {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            let len = self.animation_name.0.len();
            // There should be at least one declared value
            if len == 0 {
                return Ok(());
            }

            <%
                subproperties = "duration timing_function delay direction \
                                 fill_mode iteration_count play_state".split()
            %>

            // If any value list length is differs then we don't do a shorthand serialization
            // either.
            % for name in subproperties:
                if len != self.animation_${name}.0.len() {
                    return Ok(())
                }
            % endfor

            for i in 0..len {
                if i != 0 {
                    try!(write!(dest, ", "));
                }

                % for name in subproperties:
                    self.animation_${name}.0[i].to_css(dest)?;
                    dest.write_str(" ")?;
                % endfor
                self.animation_name.0[i].to_css(dest)?;
            }
            Ok(())
        }
    }
</%helpers:shorthand>

<%helpers:shorthand name="scroll-snap-type" products="gecko"
                    sub_properties="scroll-snap-type-x scroll-snap-type-y"
                    spec="https://drafts.csswg.org/css-scroll-snap/#propdef-scroll-snap-type">
    use properties::longhands::scroll_snap_type_x;

    pub fn parse_value(context: &ParserContext, input: &mut Parser) -> Result<Longhands, ()> {
        let result = try!(scroll_snap_type_x::parse(context, input));
        Ok(Longhands {
            scroll_snap_type_x: result,
            scroll_snap_type_y: result,
        })
    }

    impl<'a> ToCss for LonghandsToSerialize<'a>  {
        // Serializes into the single keyword value if both scroll-snap-type and scroll-snap-type-y are same.
        // Otherwise into an empty string. This is done to match Gecko's behaviour.
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            if self.scroll_snap_type_x == self.scroll_snap_type_y {
                self.scroll_snap_type_x.to_css(dest)
            } else {
                Ok(())
            }
        }
    }
</%helpers:shorthand>
