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

<%helpers:shorthand name="transition"
                    sub_properties="transition-property transition-duration transition-timing-function
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
            if property.is_none() {
                if let Ok(value) = input.try(|input| transition_property::parse_one(input)) {
                    property = Some(value);
                    continue
                }
            }

            if duration.is_none() {
                if let Ok(value) = input.try(|input| transition_duration::parse_one(input)) {
                    duration = Some(value);
                    continue
                }
            }

            if timing_function.is_none() {
                if let Ok(value) = input.try(|input| {
                    transition_timing_function::parse_one(input)
                }) {
                    timing_function = Some(value);
                    continue
                }
            }

            if delay.is_none() {
                if let Ok(value) = input.try(|input| transition_delay::parse_one(input)) {
                    delay = Some(value);
                    continue;
                }
            }

            break
        }

        if let Some(property) = property {
            Ok(SingleTransition {
                transition_property: property,
                transition_duration:
                    duration.unwrap_or(transition_duration::get_initial_single_value()),
                transition_timing_function:
                    timing_function.unwrap_or(
                        transition_timing_function::get_initial_single_value()),
                transition_delay:
                    delay.unwrap_or(transition_delay::get_initial_single_value()),
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
