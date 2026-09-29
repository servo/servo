/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::iter::Peekable;
use std::str::Chars;
use std::sync::LazyLock;

use regex::Regex;

use crate::cue::settings::{
    WebVttCue, WebVttCueSize, WebVttLineAlignment, WebVttLineAndPositionSetting,
    WebVttPositionAlignment, WebVttTextAlignment, WebVttWritingDirection,
};

fn collect_for_closure<F>(position: &mut Peekable<Chars<'_>>, f: F) -> String
where
    F: FnOnce(&char) -> bool + Copy,
{
    let mut string = String::new();
    while let Some(next) = position.next_if(f) {
        string.push(next);
    }
    string
}

pub(crate) fn collect_ascii_digits(position: &mut Peekable<Chars<'_>>) -> String {
    collect_for_closure(position, char::is_ascii_digit)
}

pub(crate) fn skip_whitespace(position: &mut Peekable<Chars<'_>>) {
    collect_for_closure(position, |c| {
        matches!(c, '\r' | '\n' | '\t' | ' ' | '\u{000C}')
    });
}

#[derive(Clone, Copy, Default, PartialEq)]
enum MostSignificantUnits {
    #[default]
    Minutes,
    Hours,
}

/// <https://w3c.github.io/webvtt/#collect-a-webvtt-timestamp>
pub(crate) fn collect_webvtt_timestamp(position: &mut Peekable<Chars<'_>>) -> Option<f64> {
    // Step 1. Let input and position be the same variables
    // as those of the same name in the algorithm that invoked these steps.
    //
    // Passed in as argument

    // Step 2. Let most significant units be minutes.
    let mut most_significant_units = MostSignificantUnits::Minutes;
    // Step 3. If position is past the end of input, return an error and abort these steps.
    // Step 4. If the character indicated by position is not an ASCII digit,
    // then return an error and abort these steps.
    if !position.peek()?.is_ascii_digit() {
        return None;
    }
    // Step 5. Collect a sequence of code points that are ASCII digits,
    // and let string be the collected substring.
    let string = collect_ascii_digits(position);
    // Step 6. Interpret string as a base-ten integer. Let value1 be that integer.
    let mut value_1 = string.parse::<f64>().ok()?;
    // Step 7. If string is not exactly two characters in length,
    // or if value1 is greater than 59, let most significant units be hours.
    if string.len() != 2 || value_1 > 59_f64 {
        most_significant_units = MostSignificantUnits::Hours;
    }
    // Step 8. If position is beyond the end of input or if the character at position is
    // not a U+003A COLON character (:), then return an error and abort these steps.
    // Otherwise, move position forwards one character.
    let _ = position.next().filter(|c| *c == '\u{003A}')?;
    // Step 9. Collect a sequence of code points that are ASCII digits,
    // and let string be the collected substring.
    let string = collect_ascii_digits(position);
    // Step 10. If string is not exactly two characters in length,
    // return an error and abort these steps.
    if string.len() != 2 {
        return None;
    }
    // Step 11. Interpret string as a base-ten integer. Let value2 be that integer.
    let mut value_2 = string.parse::<f64>().ok()?;
    // Step 12. If most significant units is hours,
    // or if position is not beyond the end of input and the character
    // at position is a U+003A COLON character (:), run these substeps:
    let value_3: f64;
    if most_significant_units == MostSignificantUnits::Hours ||
        position.peek().is_some_and(|c| *c == '\u{003A}')
    {
        // Step 12.1. If position is beyond the end of input or if
        // the character at position is not a U+003A COLON character (:),
        // then return an error and abort these steps.
        // Otherwise, move position forwards one character.
        position.next().filter(|c| *c == '\u{003A}')?;
        // Step 12.2. Collect a sequence of code points that are ASCII digits,
        // and let string be the collected substring.
        let string = collect_ascii_digits(position);
        // Step 12.3. If string is not exactly two characters in length,
        // return an error and abort these steps.
        if string.len() != 2 {
            return None;
        }
        // Step 12.4. Interpret string as a base-ten integer. Let value3 be that integer.
        value_3 = string.parse::<f64>().ok()?;
    } else {
        // Otherwise (if most significant units is not hours,
        // and either position is beyond the end of input,
        // or the character at position is not a U+003A COLON character (:)),
        // let value3 have the value of value2,
        // then value2 have the value of value1, then let value1 equal zero.
        value_3 = value_2;
        value_2 = value_1;
        value_1 = 0_f64;
    }
    // Step 13. If position is beyond the end of input or if the character at
    // position is not a U+002E FULL STOP character (.),
    // then return an error and abort these steps.
    // Otherwise, move position forwards one character.
    position.next().filter(|c| *c == '\u{002E}')?;
    // Step 14. Collect a sequence of code points that are ASCII digits,
    // and let string be the collected substring.
    let string = collect_ascii_digits(position);
    // Step 15. If string is not exactly three characters in length,
    // return an error and abort these steps.
    if string.len() != 3 {
        return None;
    }
    // Step 16. Interpret string as a base-ten integer. Let value4 be that integer.
    let value_4 = string.parse::<f64>().ok()?;
    // Step 17. If value2 is greater than 59 or if value3 is greater than 59,
    // return an error and abort these steps.
    if value_2 > 59_f64 || value_3 > 59_f64 {
        return None;
    }
    // Step 18. Let result be value1×60×60 + value2×60 + value3 + value4∕1000.
    // Step 19. Return result.
    Some(value_1 * 60_f64 * 60_f64 + value_2 * 60_f64 + value_3 + value_4 / 1000_f64)
}

/// <https://w3c.github.io/webvtt/#collect-webvtt-cue-timings-and-settings>
pub(crate) fn collect_webvtt_cue_timings_and_settings(
    identifier: String,
    input: &str,
) -> Option<WebVttCue> {
    // Step 1. Let input be the string being parsed.
    //
    // Passed in as argument

    // Step 2. Let position be a pointer into input,
    // initially pointing at the start of the string.
    let mut position = input.chars().peekable();
    // Step 3. Skip whitespace.
    skip_whitespace(&mut position);
    // Step 4. Collect a WebVTT timestamp. If that algorithm fails,
    // then abort these steps and return failure.
    // Otherwise, let cue’s text track cue start time be the collected time.
    let start_time = collect_webvtt_timestamp(position.by_ref())?;
    // Step 5. Skip whitespace.
    skip_whitespace(&mut position);
    // Step 6. If the character at position is not a U+002D HYPHEN-MINUS character (-)
    // then abort these steps and return failure.
    // Otherwise, move position forwards one character.
    let _ = position.next().filter(|c| *c == '\u{002D}')?;
    // Step 7. If the character at position is not a U+002D HYPHEN-MINUS character (-)
    // then abort these steps and return failure.
    // Otherwise, move position forwards one character.
    let _ = position.next().filter(|c| *c == '\u{002D}')?;
    // Step 8. If the character at position is not a U+003E GREATER-THAN SIGN character (>)
    // then abort these steps and return failure.
    // Otherwise, move position forwards one character.
    let _ = position.next().filter(|c| *c == '\u{003E}')?;
    // Step 9. Skip whitespace.
    skip_whitespace(&mut position);
    // Step 10. Collect a WebVTT timestamp. If that algorithm fails,
    // then abort these steps and return failure.
    // Otherwise, let cue’s text track cue end time be the collected time.
    let end_time = collect_webvtt_timestamp(position.by_ref())?;
    // Step 11. Let remainder be the trailing substring of input starting at position.
    let remainder = collect_for_closure(&mut position, |_| true);
    // Step 12. Parse the WebVTT cue settings from remainder using regions for cue.
    let cue = WebVttCue {
        identifier,
        start_time,
        end_time,
        ..Default::default()
    };
    Some(parse_the_webvtt_cue_settings(cue, remainder))
}

/// <https://w3c.github.io/webvtt/#parse-the-webvtt-cue-settings>
fn parse_the_webvtt_cue_settings(mut cue: WebVttCue, input: String) -> WebVttCue {
    // Step 1. Let settings be the result of splitting input on spaces.
    let settings = input.split_ascii_whitespace();
    // Step 2. For each token setting in the list settings, run the following substeps:
    'next_setting: for setting in settings {
        // Step 2.2. Let name be the leading substring of setting up to
        // and excluding the first U+003A COLON character (:) in that string.
        // Step 2.3. Let value be the trailing substring of setting starting from the
        // character immediately after the first U+003A COLON character (:) in that string.
        let Some((name, value)) = setting.split_once('\u{003A}') else {
            // Step 2.1. If setting does not contain a U+003A COLON character (:),
            // or if the first U+003A COLON character (:) in setting is either
            // the first or last character of setting,
            // then jump to the step labeled next setting.
            //
            // We check the first part here
            continue 'next_setting;
        };
        // Step 2.1. If setting does not contain a U+003A COLON character (:),
        // or if the first U+003A COLON character (:) in setting is either
        // the first or last character of setting,
        // then jump to the step labeled next setting.
        //
        // We check the second part here
        if name.is_empty() || value.is_empty() {
            continue 'next_setting;
        }
        // Step 2.4. Run the appropriate substeps that apply for the value of name, as follows:
        match name {
            // > If name is a case-sensitive match for "vertical"
            "vertical" => {
                // Step 2.4."vertical".1. If value is a case-sensitive match for the string "rl",
                // then let cue’s WebVTT cue writing direction be vertical growing left.
                if value == "rl" {
                    cue.writing_direction = WebVttWritingDirection::VerticalGrowingLeft;
                }
                // Step 2.4."vertical".2. Otherwise, if value is a case-sensitive match for the string "lr",
                // then let cue’s WebVTT cue writing direction be vertical growing right.
                if value == "lr" {
                    cue.writing_direction = WebVttWritingDirection::VerticalGrowingRight;
                }
                // Step 2.4."vertical".3. If cue’s WebVTT cue writing direction is not horizontal,
                // let cue’s WebVTT cue region be null (there are no vertical regions).
                // TODO
            },
            // > If name is a case-sensitive match for "line"
            "line" => {
                // Step 2.4."line".1. If value contains a U+002C COMMA character (,),
                // then let linepos be the leading substring of value up to and excluding the
                // first U+002C COMMA character (,) in that string and let linealign be the
                // trailing substring of value starting from the character immediately after the
                // first U+002C COMMA character (,) in that string.
                // Step 2.4."line".2. Otherwise let linepos be the full value string and linealign be null.
                let (linepos, linealign) = value
                    .split_once('\u{002C}')
                    .map(|(linepos, linealign)| (linepos, Some(linealign)))
                    .unwrap_or((value, None));

                // Step 2.4."line".4. If the last character in linepos is a U+0025 PERCENT SIGN character (%)
                let last_char_is_percentage =
                    linepos.chars().last().is_some_and(|c| c == '\u{0025}');
                let number = if last_char_is_percentage {
                    // If parse a percentage string from linepos doesn’t fail,
                    // let number be the returned percentage, otherwise jump to the step labeled next setting.
                    let Some(number) = parse_a_percentage_string(linepos) else {
                        continue 'next_setting;
                    };
                    number
                } else {
                    let mut chars = linepos.chars().peekable();
                    let mut has_at_least_one_dot = false;
                    let mut last_char: Option<char> = None;
                    let mut at_least_one_digit = false;
                    while let Some(current_char) = chars.next() {
                        match current_char {
                            // Step 2.4."line".4.2. If any character in linepos other than the first character is
                            // a U+002D HYPHEN-MINUS character (-), then jump to the step labeled next setting.
                            '\u{002D}' => {
                                if last_char.is_some() {
                                    continue 'next_setting;
                                }
                            },
                            '\u{002E}' => {
                                // Step 2.4."line".4.3. If there are more than one U+002E DOT characters (.),
                                // then jump to the step labeled next setting.
                                if has_at_least_one_dot {
                                    continue 'next_setting;
                                }
                                has_at_least_one_dot = true;
                                // Step 2.4."line".4.4. If there is a U+002E DOT character (.)
                                // and the character before or the character after is not an ASCII digit,
                                // or if the U+002E DOT character (.) is the first or the last character,
                                // then jump to the step labeled next setting.
                                if last_char.is_none_or(|c| !c.is_ascii_digit()) ||
                                    chars.peek().is_none_or(|c| !c.is_ascii_digit())
                                {
                                    continue 'next_setting;
                                }
                            },
                            _ => {
                                // Step 2.4."line".4.1. If linepos contains any characters other than
                                // U+002D HYPHEN-MINUS characters (-), ASCII digits, and U+002E DOT character (.),
                                // then jump to the step labeled next setting.
                                if !current_char.is_ascii_digit() {
                                    continue 'next_setting;
                                }
                                at_least_one_digit = true;
                            },
                        }
                        last_char = Some(current_char);
                    }
                    // Step 2.4."line".3. If linepos does not contain at least one ASCII digit,
                    // then jump to the step labeled next setting.
                    if !at_least_one_digit {
                        continue 'next_setting;
                    }
                    // Step 2.4."line".4.5. Let number be the result of parsing linepos using the
                    // rules for parsing floating-point number values. [HTML]
                    let Some(number) = linepos.parse::<f64>().ok() else {
                        // Step 2.4."line".4.6. If number is an error,
                        // then jump to the step labeled next setting.
                        continue 'next_setting;
                    };
                    number
                };
                match linealign {
                    // Step 2.4."line".5. If linealign is a case-sensitive match for the string "start",
                    // then let cue’s WebVTT cue line alignment be start alignment.
                    Some("start") => {
                        cue.line_alignment = WebVttLineAlignment::Start;
                    },
                    // Step 2.4."line".6. If linealign is a case-sensitive match for the string "center",
                    // then let cue’s WebVTT cue line alignment be center alignment.
                    Some("center") => {
                        cue.line_alignment = WebVttLineAlignment::Center;
                    },
                    // Step 2.4."line".7. If linealign is a case-sensitive match for the string "end",
                    // then let cue’s WebVTT cue line alignment be end alignment.
                    Some("end") => {
                        cue.line_alignment = WebVttLineAlignment::End;
                    },
                    // Step 2.4."line".8. Otherwise, if linealign is not null,
                    // then jump to the step labeled next setting.
                    Some(_) => {
                        continue 'next_setting;
                    },
                    _ => {},
                }
                // Step 2.4."line".9. Let cue’s WebVTT cue line be number.
                cue.line = WebVttLineAndPositionSetting::Double(number);
                // Step 2.4."line".10. If the last character in linepos is a U+0025 PERCENT SIGN character (%),
                // then let cue’s WebVTT cue snap-to-lines flag be false. Otherwise, let it be true.
                cue.snap_to_lines = (!last_char_is_percentage).into();
                // If cue’s WebVTT cue line is not auto,
                // let cue’s WebVTT cue region be null
                // (the cue has been explicitly positioned with a line offset
                // and thus drops out of the region).
                // TODO
            },
            // > If name is a case-sensitive match for "position"
            "position" => {
                // Step 2.4."position".1. If value contains a U+002C COMMA character (,),
                // then let colpos be the leading substring of value up to and excluding the
                // first U+002C COMMA character (,) in that string and let colalign be the
                // trailing substring of value starting from the character immediately after the
                // first U+002C COMMA character (,) in that string.
                // Step 2.4."position".2. Otherwise let colpos be the full value string and colalign be null.
                let (colpos, colalign) = value
                    .split_once('\u{002C}')
                    .map(|(colpos, colalign)| (colpos, Some(colalign)))
                    .unwrap_or((value, None));
                // Step 2.4."position".3. If parse a percentage string from colpos doesn’t fail,
                // let number be the returned percentage,
                // otherwise jump to the step labeled next setting
                // (position’s value remains the special value auto).
                let Some(number) = parse_a_percentage_string(colpos) else {
                    continue 'next_setting;
                };
                match colalign {
                    // Step 2.4."position".4. If colalign is a case-sensitive match for the string "line-left",
                    // then let cue’s WebVTT cue position alignment be line-left alignment.
                    Some("line-left") => {
                        cue.position_alignment = WebVttPositionAlignment::LineLeft;
                    },
                    // Step 2.4."position".5. Otherwise, if colalign is a case-sensitive match for the string "center",
                    // then let cue’s WebVTT cue position alignment be center alignment.
                    Some("center") => {
                        cue.position_alignment = WebVttPositionAlignment::Center;
                    },
                    // Step 2.4."position".6. Otherwise, if colalign is a case-sensitive match for the string "line-right",
                    // then let cue’s WebVTT cue position alignment be line-right alignment.
                    Some("line-right") => {
                        cue.position_alignment = WebVttPositionAlignment::LineRight;
                    },
                    // Step 2.4."position".7. Otherwise, if colalign is not null,
                    // then jump to the step labeled next setting.
                    Some(_) => {
                        continue 'next_setting;
                    },
                    _ => {},
                }
                // Step 2.4."position".8. Let cue’s position be number.
                cue.position = WebVttLineAndPositionSetting::Double(number);
            },
            // > If name is a case-sensitive match for "size"
            "size" => {
                // Step 2.4."size".1. If parse a percentage string from value doesn’t fail,
                // let number be the returned percentage,
                // otherwise jump to the step labeled next setting.
                let Some(number) = parse_a_percentage_string(value) else {
                    continue 'next_setting;
                };
                // Step 2.4."size".2. Let cue’s WebVTT cue size be number.
                cue.size = WebVttCueSize(number);
                // Step 2.4."size".3. If cue’s WebVTT cue size is not 100,
                // let cue’s WebVTT cue region be null
                // (the cue has been explicitly sized and thus drops out of the region).
                // TODO
            },
            // > If name is a case-sensitive match for "align"
            "align" => {
                // Step 2.4."align".1. If value is a case-sensitive match for the string "start",
                // then let cue’s WebVTT cue text alignment be start alignment.
                if value == "start" {
                    cue.text_alignment = WebVttTextAlignment::Start;
                }
                // Step 2.4."align".2. If value is a case-sensitive match for the string "center",
                // then let cue’s WebVTT cue text alignment be center alignment.
                if value == "center" {
                    cue.text_alignment = WebVttTextAlignment::Center;
                }
                // Step 2.4."align".3. If value is a case-sensitive match for the string "end",
                // then let cue’s WebVTT cue text alignment be end alignment.
                if value == "end" {
                    cue.text_alignment = WebVttTextAlignment::End;
                }
                // Step 2.4."align".4. If value is a case-sensitive match for the string "left",
                // then let cue’s WebVTT cue text alignment be left alignment.
                if value == "left" {
                    cue.text_alignment = WebVttTextAlignment::Left;
                }
                // Step 2.4."align".5. If value is a case-sensitive match for the string "right",
                // then let cue’s WebVTT cue text alignment be right alignment.
                if value == "right" {
                    cue.text_alignment = WebVttTextAlignment::Right;
                }
            },
            _ => {},
        }
    }
    cue
}

/// <https://w3c.github.io/webvtt/#webvtt-percentage>
static WEB_VTT_PERCENTAGE_GRAMMAR: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"^(?P<number>[0-9]+(\.[0-9]+)?)%$"#).unwrap());

/// <https://w3c.github.io/webvtt/#parse-a-percentage-string>
fn parse_a_percentage_string(input: &str) -> Option<f64> {
    // Step 1. Let input be the string being parsed.
    //
    // Passed in as argument

    // Step 2. If input does not match the syntax for a WebVTT percentage, then fail.
    let captures = WEB_VTT_PERCENTAGE_GRAMMAR.captures(input)?;
    // Step 3. Remove the last character from input.
    let input = captures.name("number").expect("Must always have a capture");
    // Step 4. Let percentage be the result of parsing input using the rules for parsing floating-point number values. [HTML]
    // Step 5. If percentage is an error, is less than 0, or is greater than 100, then fail.
    // Step 6. Return percentage.
    input
        .as_str()
        .trim()
        .parse::<f64>()
        .ok()
        .filter(|percentage| *percentage >= 0. && *percentage <= 100.)
}

#[cfg(test)]
mod tests {
    use crate::collectors::collect_webvtt_timestamp;
    use crate::shared_test_setup::compute_result_in_seconds;

    fn parse_timestamp(input: &str) -> Option<f64> {
        collect_webvtt_timestamp(&mut input.chars().peekable())
    }

    #[test]
    fn test_parses_start_timestamp_correctly() {
        assert_eq!(
            parse_timestamp("01:10:03.000"),
            Some(compute_result_in_seconds(1., 10., 3., 0.))
        );
    }

    #[test]
    fn test_parses_maximum_minute_timestamp() {
        assert_eq!(
            parse_timestamp("10:59:03.000"),
            Some(compute_result_in_seconds(10., 59., 3., 0.))
        );
    }

    #[test]
    fn test_parses_maximum_second_timestamp() {
        assert_eq!(
            parse_timestamp("10:04:59.000"),
            Some(compute_result_in_seconds(10., 4., 59., 0.))
        );
    }

    #[test]
    fn test_hours_more_than_59_ensures_three_values() {
        assert_eq!(parse_timestamp("60:04.000"), None);
    }

    #[test]
    fn test_first_value_below_60_implies_minutes() {
        assert_eq!(
            parse_timestamp("59:04.000"),
            Some(compute_result_in_seconds(0., 59., 4., 0.))
        );
    }

    #[test]
    fn test_seconds_cannot_exceed_59() {
        assert_eq!(parse_timestamp("05:60.000"), None);
    }

    #[test]
    fn test_minutes_cannot_exceed_59() {
        assert_eq!(parse_timestamp("05:60:35.000"), None);
    }

    #[test]
    fn test_hours_can_exceed_59() {
        assert_eq!(
            parse_timestamp("60:20:35.000"),
            Some(compute_result_in_seconds(60., 20., 35., 0.))
        );
    }

    #[test]
    fn test_parses_handles_exact_hour_correctly() {
        assert_eq!(
            parse_timestamp("01:00:00.000"),
            Some(compute_result_in_seconds(1., 0., 0., 0.))
        );
    }
}
