/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

<%namespace name="helpers" file="/helpers.mako.rs" />

<%helpers:shorthand name="overflow" sub_properties="overflow-x overflow-y">
    use properties::longhands::{overflow_x, overflow_y};

    let overflow = try!(overflow_x::parse(context, input));
    Ok(Longhands {
        overflow_x: Some(overflow),
        overflow_y: Some(overflow_y::SpecifiedValue(overflow)),
    })
</%helpers:shorthand>

macro_rules! try_parse_one {
    ($input: expr, $var: ident, $prop_module: ident) => {
        if $var.is_none() {
            if let Ok(value) = $input.try($prop_module::parse_one) {
                $var = Some(value);
                continue;
            }
        }
    }
}

<%helpers:shorthand name="transition"
                    sub_properties="transition-property transition-duration
                                    transition-timing-function
                                    transition-delay">
    use properties::longhands::{transition_delay, transition_duration, transition_property};
    use properties::longhands::{transition_timing_function};

    struct SingleTransition {
        transition_property: transition_property::SingleSpecifiedValue,
        transition_duration: transition_duration::SingleSpecifiedValue,
        transition_timing_function: transition_timing_function::SingleSpecifiedValue,
        transition_delay: transition_delay::SingleSpecifiedValue,
    }

    fn parse_one_transition(input: &mut Parser) -> Result<SingleTransition,()> {
        let (mut property, mut duration) = (None, None);
        let (mut timing_function, mut delay) = (None, None);
        loop {
            try_parse_one!(input, property, transition_property);
            try_parse_one!(input, duration, transition_duration);
            try_parse_one!(input, timing_function, transition_timing_function);
            try_parse_one!(input, delay, transition_delay);

            break
        }

        if let Some(property) = property {
            Ok(SingleTransition {
                transition_property: property,
                transition_duration:
                    duration.unwrap_or_else(transition_duration::get_initial_single_value),
                transition_timing_function:
                    timing_function.unwrap_or_else(
                        transition_timing_function::get_initial_single_value),
                transition_delay:
                    delay.unwrap_or_else(transition_delay::get_initial_single_value),
            })
        } else {
            Err(())
        }
    }

    if input.try(|input| input.expect_ident_matching("none")).is_ok() {
        return Ok(Longhands {
            transition_property: None,
            transition_duration: None,
            transition_timing_function: None,
            transition_delay: None,
        })
    }

    let results = try!(input.parse_comma_separated(parse_one_transition));
    let (mut properties, mut durations) = (Vec::new(), Vec::new());
    let (mut timing_functions, mut delays) = (Vec::new(), Vec::new());
    for result in results {
        properties.push(result.transition_property);
        durations.push(result.transition_duration);
        timing_functions.push(result.transition_timing_function);
        delays.push(result.transition_delay);
    }

    Ok(Longhands {
        transition_property: Some(transition_property::SpecifiedValue(properties)),
        transition_duration: Some(transition_duration::SpecifiedValue(durations)),
        transition_timing_function:
            Some(transition_timing_function::SpecifiedValue(timing_functions)),
        transition_delay: Some(transition_delay::SpecifiedValue(delays)),
    })
</%helpers:shorthand>

<%helpers:shorthand name="animation"
                    sub_properties="animation-name animation-duration
                                    animation-timing-function animation-delay
                                    animation-iteration-count animation-direction
                                    animation-fill-mode animation-play-state">
    use properties::longhands::{animation_name, animation_duration, animation_timing_function};
    use properties::longhands::{animation_delay, animation_iteration_count, animation_direction};
    use properties::longhands::{animation_fill_mode, animation_play_state};

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

    fn parse_one_animation(input: &mut Parser) -> Result<SingleAnimation,()> {
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
            try_parse_one!(input, duration, animation_duration);
            try_parse_one!(input, timing_function, animation_timing_function);
            try_parse_one!(input, delay, animation_delay);
            try_parse_one!(input, iteration_count, animation_iteration_count);
            try_parse_one!(input, direction, animation_direction);
            try_parse_one!(input, fill_mode, animation_fill_mode);
            try_parse_one!(input, play_state, animation_play_state);
            try_parse_one!(input, name, animation_name);

            break
        }

        if let Some(name) = name {
            Ok(SingleAnimation {
                animation_name: name,
                animation_duration:
                    duration.unwrap_or_else(animation_duration::get_initial_single_value),
                animation_timing_function:
                    timing_function.unwrap_or_else(animation_timing_function::get_initial_single_value),
                animation_delay:
                    delay.unwrap_or_else(animation_delay::get_initial_single_value),
                animation_iteration_count:
                    iteration_count.unwrap_or_else(animation_iteration_count::get_initial_single_value),
                animation_direction:
                    direction.unwrap_or_else(animation_direction::get_initial_single_value),
                animation_fill_mode:
                    fill_mode.unwrap_or_else(animation_fill_mode::get_initial_single_value),
                animation_play_state:
                    play_state.unwrap_or_else(animation_play_state::get_initial_single_value),
            })
        } else {
            Err(())
        }
    }

    if input.try(|input| input.expect_ident_matching("none")).is_ok() {
        return Ok(Longhands {
            animation_name: None,
            animation_duration: None,
            animation_timing_function: None,
            animation_delay: None,
            animation_iteration_count: None,
            animation_direction: None,
            animation_fill_mode: None,
            animation_play_state: None,
        })
    }

    let results = try!(input.parse_comma_separated(parse_one_animation));

    let mut names = vec![];
    let mut durations = vec![];
    let mut timing_functions = vec![];
    let mut delays = vec![];
    let mut iteration_counts = vec![];
    let mut directions = vec![];
    let mut fill_modes = vec![];
    let mut play_states = vec![];

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

    Ok(Longhands {
        animation_name: Some(animation_name::SpecifiedValue(names)),
        animation_duration: Some(animation_duration::SpecifiedValue(durations)),
        animation_timing_function: Some(animation_timing_function::SpecifiedValue(timing_functions)),
        animation_delay: Some(animation_delay::SpecifiedValue(delays)),
        animation_iteration_count: Some(animation_iteration_count::SpecifiedValue(iteration_counts)),
        animation_direction: Some(animation_direction::SpecifiedValue(directions)),
        animation_fill_mode: Some(animation_fill_mode::SpecifiedValue(fill_modes)),
        animation_play_state: Some(animation_play_state::SpecifiedValue(play_states)),
    })
</%helpers:shorthand>
