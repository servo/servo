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
    % for prop in "delay duration property timing_function".split():
    use properties::longhands::transition_${prop};
    % endfor

    pub fn parse_value(context: &ParserContext, input: &mut Parser) -> Result<Longhands, ()> {
        struct SingleTransition {
            % for prop in "property duration timing_function delay".split():
            transition_${prop}: transition_${prop}::SingleSpecifiedValue,
            % endfor
        }

        fn parse_one_transition(context: &ParserContext, input: &mut Parser) -> Result<SingleTransition,()> {
            % for prop in "property duration timing_function delay".split():
            let mut ${prop} = None;
            % endfor

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
                    % for prop in "duration timing_function delay".split():
                    transition_${prop}: ${prop}.unwrap_or_else(transition_${prop}::single_value
                                                                                 ::get_initial_specified_value),
                    % endfor
                })
            } else {
                Err(())
            }
        }

        % for prop in "property duration timing_function delay".split():
        let mut ${prop}s = Vec::new();
        % endfor

        if input.try(|input| input.expect_ident_matching("none")).is_err() {
            let results = try!(input.parse_comma_separated(|i| parse_one_transition(context, i)));
            for result in results {
                % for prop in "property duration timing_function delay".split():
                ${prop}s.push(result.transition_${prop});
                % endfor
            }
        } else {
            // `transition: none` is a valid syntax, and we keep transition_property empty because |none| is not
            // a valid TransitionProperty.
            // durations, delays, and timing_functions are not allowed as empty, so before we convert them into
            // longhand properties, we need to put initial values for none transition.
            % for prop in "duration timing_function delay".split():
            ${prop}s.push(transition_${prop}::single_value::get_initial_specified_value());
            % endfor
        }

        Ok(Longhands {
            % for prop in "property duration timing_function delay".split():
            transition_${prop}: transition_${prop}::SpecifiedValue(${prop}s),
            % endfor
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
    <%
        props = "name duration timing_function delay iteration_count \
                 direction fill_mode play_state".split()
    %>
    use parser::Parse;
    % for prop in props:
    use properties::longhands::animation_${prop};
    % endfor

    pub fn parse_value(context: &ParserContext, input: &mut Parser) -> Result<Longhands, ()> {
        struct SingleAnimation {
            % for prop in props:
            animation_${prop}: animation_${prop}::SingleSpecifiedValue,
            % endfor
        }

        fn parse_one_animation(context: &ParserContext, input: &mut Parser) -> Result<SingleAnimation,()> {
            % for prop in props:
            let mut ${prop} = None;
            % endfor

            let mut parsed = 0;
            // NB: Name must be the last one here so that keywords valid for other
            // longhands are not interpreted as names.
            //
            // Also, duration must be before delay, see
            // https://drafts.csswg.org/css-animations/#typedef-single-animation
            loop {
                parsed += 1;
                try_parse_one!(context, input, duration, animation_duration);
                try_parse_one!(context, input, timing_function, animation_timing_function);
                try_parse_one!(context, input, delay, animation_delay);
                try_parse_one!(context, input, iteration_count, animation_iteration_count);
                try_parse_one!(input, direction, animation_direction);
                try_parse_one!(input, fill_mode, animation_fill_mode);
                try_parse_one!(input, play_state, animation_play_state);
                try_parse_one!(context, input, name, animation_name);

                parsed -= 1;
                break
            }

            // If nothing is parsed, this is an invalid entry.
            if parsed == 0 {
                Err(())
            } else {
                Ok(SingleAnimation {
                    % for prop in props:
                    animation_${prop}: ${prop}.unwrap_or_else(animation_${prop}::single_value
                                                              ::get_initial_specified_value),
                    % endfor
                })
            }
        }

        % for prop in props:
        let mut ${prop}s = vec![];
        % endfor

        let results = try!(input.parse_comma_separated(|i| parse_one_animation(context, i)));
        for result in results.into_iter() {
            % for prop in props:
            ${prop}s.push(result.animation_${prop});
            % endfor
        }

        Ok(Longhands {
            % for prop in props:
            animation_${prop}: animation_${prop}::SpecifiedValue(${prop}s),
            % endfor
        })
    }

    impl<'a> ToCss for LonghandsToSerialize<'a>  {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            let len = self.animation_name.0.len();
            // There should be at least one declared value
            if len == 0 {
                return Ok(());
            }

            // If any value list length is differs then we don't do a shorthand serialization
            // either.
            % for name in props[1:]:
                if len != self.animation_${name}.0.len() {
                    return Ok(())
                }
            % endfor

            for i in 0..len {
                if i != 0 {
                    try!(write!(dest, ", "));
                }

                % for name in props[1:]:
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
