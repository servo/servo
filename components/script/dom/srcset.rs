/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::sync::LazyLock;

use cssparser::{Parser, ParserInput};
use regex::Regex;
use style::attr::parse_unsigned_integer;
use style::stylesheets::CssRuleType;
use style::values::specified::source_size_list::SourceSizeList;
use style_traits::ParsingMode;
use url::Url;

use crate::css::parser_context_for_anonymous_content;

/// <https://html.spec.whatwg.org/multipage/#source-set>
#[derive(MallocSizeOf)]
pub(crate) struct SourceSet {
    pub image_sources: Vec<ImageSource>,
    pub source_size: SourceSizeList,
}

impl SourceSet {
    pub fn new() -> SourceSet {
        SourceSet {
            image_sources: Vec::new(),
            source_size: SourceSizeList::empty(),
        }
    }
}

#[derive(Clone, Debug, MallocSizeOf, PartialEq)]
pub struct ImageSource {
    pub url: String,
    pub descriptor: Descriptor,
}

#[derive(Clone, Debug, MallocSizeOf, PartialEq)]
pub struct Descriptor {
    pub width: Option<u32>,
    pub density: Option<f64>,
}

#[derive(Clone, Copy, Debug)]
enum ParseState {
    InDescriptor,
    InParens,
    AfterDescriptor,
}

/// <https://html.spec.whatwg.org/multipage/#parse-a-sizes-attribute>
pub fn parse_a_sizes_attribute(value: &str) -> SourceSizeList {
    let mut input = ParserInput::new(value);
    let mut parser = Parser::new(&mut input);
    let url_data = Url::parse("about:blank").unwrap().into();
    // FIXME(emilio): why ::empty() instead of ::DEFAULT? Also, what do
    // browsers do regarding quirks-mode in a media list?
    let context =
        parser_context_for_anonymous_content(CssRuleType::Style, ParsingMode::empty(), &url_data);
    SourceSizeList::parse(&context, &mut parser)
}

/// Collect sequence of code points
/// <https://infra.spec.whatwg.org/#collect-a-sequence-of-code-points>
pub(crate) fn collect_sequence_characters(
    s: &str,
    mut predicate: impl FnMut(&char) -> bool,
) -> (&str, &str) {
    let i = s.find(|ch| !predicate(&ch)).unwrap_or(s.len());
    (&s[0..i], &s[i..])
}

/// <https://html.spec.whatwg.org/multipage/#valid-non-negative-integer>
/// TODO(#39315): Use the validation rule from Stylo
fn is_valid_non_negative_integer_string(s: &str) -> bool {
    s.chars().all(|c| c.is_ascii_digit())
}

/// <https://html.spec.whatwg.org/multipage/#valid-floating-point-number>
/// TODO(#39315): Use the validation rule from Stylo
fn is_valid_floating_point_number_string(s: &str) -> bool {
    static RE: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"^-?(?:\d+\.\d+|\d+|\.\d+)(?:(e|E)(\+|\-)?\d+)?$").unwrap());

    RE.is_match(s)
}

/// Parse an `srcset` attribute:
/// <https://html.spec.whatwg.org/multipage/#parsing-a-srcset-attribute>.
pub fn parse_a_srcset_attribute(input: &str) -> Vec<ImageSource> {
    // > 1. Let input be the value passed to this algorithm.
    // > 2. Let position be a pointer into input, initially pointing at the start of the string.
    let mut current_index = 0;

    // > 3. Let candidates be an initially empty source set.
    let mut candidates = vec![];
    while current_index < input.len() {
        let remaining_string = &input[current_index..];

        // > 4. Splitting loop: Collect a sequence of code points that are ASCII whitespace or
        // > U+002C COMMA characters from input given position. If any U+002C COMMA
        // > characters were collected, that is a parse error.
        // NOTE: A parse error indicating a non-fatal mismatch between the input and the
        // requirements will be silently ignored to match the behavior of other browsers.
        // <https://html.spec.whatwg.org/multipage/#concept-microsyntax-parse-error>
        let (collected_characters, string_after_whitespace) =
            collect_sequence_characters(remaining_string, |character| {
                *character == ',' || character.is_ascii_whitespace()
            });

        // Add the length of collected whitespace, to find the start of the URL we are going
        // to parse.
        current_index += collected_characters.len();

        // > 5. If position is past the end of input, return candidates.
        if string_after_whitespace.is_empty() {
            return candidates;
        }

        // 6. Collect a sequence of code points that are not ASCII whitespace from input
        // given position, and let that be url.
        let (url, _) =
            collect_sequence_characters(string_after_whitespace, |c| !char::is_ascii_whitespace(c));

        // Add the length of `url` that we will parse to advance the index of the next part
        // of the string to prase.
        current_index += url.len();

        // 7. Let descriptors be a new empty list.
        let mut descriptors = Vec::new();

        // > 8. If url ends with U+002C (,), then:
        // >    1. Remove all trailing U+002C COMMA characters from url. If this removed
        // >       more than one character, that is a parse error.
        if url.ends_with(',') {
            let image_source = ImageSource {
                url: url.trim_end_matches(',').into(),
                descriptor: Descriptor {
                    width: None,
                    density: None,
                },
            };
            candidates.push(image_source);
            continue;
        }

        // Otherwise:
        // > 8.1. Descriptor tokenizer: Skip ASCII whitespace within input given position.
        let descriptors_string = &input[current_index..];
        let (spaces, descriptors_string) =
            collect_sequence_characters(descriptors_string, |character| {
                character.is_ascii_whitespace()
            });
        current_index += spaces.len();

        // > 8.2. Let current descriptor be the empty string.
        let mut current_descriptor = String::new();

        // > 8.3. Let state be "in descriptor".
        let mut state = ParseState::InDescriptor;

        // > 8.4. Let c be the character at position. Do the following depending on the value of
        // > state. For the purpose of this step, "EOF" is a special character representing
        // > that position is past the end of input.
        let mut characters = descriptors_string.chars();
        let mut character = characters.next();
        if let Some(character) = character {
            current_index += character.len_utf8();
        }

        loop {
            match (state, character) {
                (ParseState::InDescriptor, Some(character)) if character.is_ascii_whitespace() => {
                    // > If current descriptor is not empty, append current descriptor to
                    // > descriptors and let current descriptor be the empty string. Set
                    // > state to after descriptor.
                    if !current_descriptor.is_empty() {
                        descriptors.push(current_descriptor);
                        current_descriptor = String::new();
                        state = ParseState::AfterDescriptor;
                    }
                },
                (ParseState::InDescriptor, Some(',')) => {
                    // > Advance position to the next character in input. If current descriptor
                    // > is not empty, append current descriptor to descriptors. Jump to the
                    // > step labeled descriptor parser.
                    if !current_descriptor.is_empty() {
                        descriptors.push(current_descriptor);
                    }
                    break;
                },
                (ParseState::InDescriptor, Some('(')) => {
                    // > Append c to current descriptor. Set state to in parens.
                    current_descriptor.push('(');
                    state = ParseState::InParens;
                },
                (ParseState::InDescriptor, Some(character)) => {
                    // > Append c to current descriptor.
                    current_descriptor.push(character);
                },
                (ParseState::InDescriptor, None) => {
                    // > If current descriptor is not empty, append current descriptor to
                    // > descriptors. Jump to the step labeled descriptor parser.
                    if !current_descriptor.is_empty() {
                        descriptors.push(current_descriptor);
                    }
                    break;
                },
                (ParseState::InParens, Some(')')) => {
                    // > Append c to current descriptor. Set state to in descriptor.
                    current_descriptor.push(')');
                    state = ParseState::InDescriptor;
                },
                (ParseState::InParens, Some(character)) => {
                    // Append c to current descriptor.
                    current_descriptor.push(character);
                },
                (ParseState::InParens, None) => {
                    // > Append current descriptor to descriptors. Jump to the step
                    // > labeled descriptor parser.
                    descriptors.push(current_descriptor);
                    break;
                },
                (ParseState::AfterDescriptor, Some(character))
                    if character.is_ascii_whitespace() =>
                {
                    // > Stay in this state.
                },
                (ParseState::AfterDescriptor, Some(_)) => {
                    // > Set state to in descriptor. Set position to the previous
                    // > character in input.
                    state = ParseState::InDescriptor;
                    continue;
                },
                (ParseState::AfterDescriptor, None) => {
                    // > Jump to the step labeled descriptor parser.
                    break;
                },
            }

            character = characters.next();
            if let Some(character) = character {
                current_index += character.len_utf8();
            }
        }

        // > 9. Descriptor parser: Let error be no.
        let mut error = false;
        // > 10. Let width be absent.
        let mut width: Option<u32> = None;
        // > 11. Let density be absent.
        let mut density: Option<f64> = None;
        // > 12. Let future-compat-h be absent.
        let mut future_compat_h: Option<u32> = None;

        // > 13. For each descriptor in descriptors, run the appropriate set of steps from
        // > the following list:
        for descriptor in descriptors.into_iter() {
            let Some(last_character) = descriptor.chars().last() else {
                break;
            };

            let first_part_of_string = &descriptor[0..descriptor.len() - last_character.len_utf8()];
            match last_character {
                // > If the descriptor consists of a valid non-negative integer followed by a
                // > U+0077 LATIN SMALL LETTER W character
                // > 1. If the user agent does not support the sizes attribute, let error be yes.
                // > 2. If width and density are not both absent, then let error be yes.
                // > 3. Apply the rules for parsing non-negative integers to the descriptor.
                // >    If the result is 0, let error be yes. Otherwise, let width be the result.
                'w' if is_valid_non_negative_integer_string(first_part_of_string) &&
                    density.is_none() &&
                    width.is_none() =>
                {
                    match parse_unsigned_integer(first_part_of_string.chars()) {
                        Ok(number) if number > 0 => {
                            width = Some(number);
                            continue;
                        },
                        _ => error = true,
                    }
                },

                // > If the descriptor consists of a valid floating-point number followed by a
                // > U+0078 LATIN SMALL LETTER X character
                // > 1. If width, density and future-compat-h are not all absent, then let
                // >    error be yes.
                // > 2. Apply the rules for parsing floating-point number values to the
                // >    descriptor. If the result is less than 0, let error be yes. Otherwise, let
                // >    density be the result.
                //
                // The HTML specification has a procedure for parsing floats that is different enough from
                // the one that stylo uses, that it's better to use Rust's float parser here. This is
                // what Gecko does, but it also checks to see if the number is a valid HTML-spec compliant
                // number first. Not doing that means that we might be parsing numbers that otherwise
                // wouldn't parse.
                'x' if is_valid_floating_point_number_string(first_part_of_string) &&
                    width.is_none() &&
                    density.is_none() &&
                    future_compat_h.is_none() =>
                {
                    match first_part_of_string.parse::<f64>() {
                        Ok(number) if number.is_finite() && number >= 0. => {
                            density = Some(number);
                            continue;
                        },
                        _ => error = true,
                    }
                },

                // > If the descriptor consists of a valid non-negative integer followed by a
                // > U+0068 LATIN SMALL LETTER H character
                // >   This is a parse error.
                // > 1. If future-compat-h and density are not both absent, then let error be
                // >    yes.
                // > 2. Apply the rules for parsing non-negative integers to the descriptor.
                // >    If the result is 0, let error be yes. Otherwise, let future-compat-h be the
                // >    result.
                'h' if is_valid_non_negative_integer_string(first_part_of_string) &&
                    future_compat_h.is_none() &&
                    density.is_none() =>
                {
                    match parse_unsigned_integer(first_part_of_string.chars()) {
                        Ok(number) if number > 0 => {
                            future_compat_h = Some(number);
                            continue;
                        },
                        _ => error = true,
                    }
                },

                // > Anything else
                // >  Let error be yes.
                _ => error = true,
            }

            if error {
                break;
            }
        }

        // > 14. If future-compat-h is not absent and width is absent, let error be yes.
        if future_compat_h.is_some() && width.is_none() {
            error = true;
        }

        // Step 15. If error is still no, then append a new image source to candidates whose URL is
        // url, associated with a width width if not absent and a pixel density density if not
        // absent. Otherwise, there is a parse error.
        if !error {
            let image_source = ImageSource {
                url: url.into(),
                descriptor: Descriptor { width, density },
            };
            candidates.push(image_source);
        }

        // Step 16. Return to the step labeled splitting loop.
    }
    candidates
}
