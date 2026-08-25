/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::sync::LazyLock;

use app_units::Au;
use cssparser::{Parser, ParserInput};
use regex::Regex;
use rustc_hash::FxHashSet;
use script_bindings::codegen::GenericBindings::NodeBinding::NodeMethods;
use script_bindings::inheritance::Castable;
use script_bindings::root::DomRoot;
use script_bindings::str::USVString;
use style::attr::parse_unsigned_integer;
use style::stylesheets::CssRuleType;
use style::values::specified::source_size_list::SourceSizeList;
use style_traits::ParsingMode;
use xml5ever::local_name;

use crate::css::css::{ANONYMOUS_CONTENT_URL_DATA, parser_context_for_anonymous_content};
use crate::dom::htmlimageelement::HTMLImageElement;
use crate::dom::htmllinkelement::HTMLLinkElement;
use crate::dom::htmlpictureelement::HTMLPictureElement;
use crate::dom::htmlsourceelement::HTMLSourceElement;
use crate::dom::medialist::MediaList;
use crate::dom::node::NodeTraits;
use crate::dom::{Document, Element, Node};

/// Supported image MIME types as defined by
/// <https://mimesniff.spec.whatwg.org/#image-mime-type>.
/// Keep this in sync with 'detect_image_format' from components/pixels/lib.rs
const SUPPORTED_IMAGE_MIME_TYPES: &[&str] = &[
    "image/bmp",
    "image/gif",
    "image/jpeg",
    "image/jpg",
    "image/pjpeg",
    "image/png",
    "image/apng",
    "image/x-png",
    "image/svg+xml",
    "image/vnd.microsoft.icon",
    "image/x-icon",
    "image/webp",
];

/// <https://html.spec.whatwg.org/multipage/#source-set>
#[derive(Clone, Debug, MallocSizeOf)]
pub(crate) struct SourceSet {
    pub image_sources: Vec<ImageSource>,
    pub source_size: SourceSizeList,
}

/// <https://html.spec.whatwg.org/multipage/#image-source>
#[derive(Clone, Debug, MallocSizeOf, PartialEq)]
pub struct ImageSource {
    pub url: String,
    pub descriptor: Descriptor,
}

/// <https://html.spec.whatwg.org/multipage/#width-descriptor>
/// <https://html.spec.whatwg.org/multipage/#pixel-density-descriptor>
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

impl SourceSet {
    pub fn new() -> SourceSet {
        SourceSet {
            image_sources: Vec::new(),
            source_size: SourceSizeList::empty(),
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#create-a-source-set>
    pub fn create_source_set(
        default_source: &str,
        srcset: &str,
        sizes: &str,
        document: &Document,
    ) -> SourceSet {
        // Step 1. Let source set be an empty source set.
        let mut source_set = SourceSet::new();

        // Step 2. If srcset is not an empty string, then set source set to the result of parsing
        // srcset.
        if !srcset.is_empty() {
            source_set.image_sources = parse_a_srcset_attribute(srcset);
        }

        // Step 3. Set source set's source size to the result of parsing sizes with img.
        if !sizes.is_empty() {
            source_set.source_size = parse_a_sizes_attribute(sizes);
        }

        // Step 4. If default source is not the empty string and source set does not contain an
        // image source with a pixel density descriptor value of 1, and no image source with a width
        // descriptor, append default source to source set.
        let no_density_source_of_1 = source_set
            .image_sources
            .iter()
            .all(|source| source.descriptor.density != Some(1.));
        let no_width_descriptor = source_set
            .image_sources
            .iter()
            .all(|source| source.descriptor.width.is_none());
        if !default_source.is_empty() && no_density_source_of_1 && no_width_descriptor {
            source_set.image_sources.push(ImageSource {
                url: String::from(default_source),
                descriptor: Descriptor {
                    width: None,
                    density: None,
                },
            })
        }

        // Step 5. Normalize the source densities of source set.
        source_set.normalise_source_densities(document);

        // Step 6. Return source set.
        source_set
    }

    /// <https://html.spec.whatwg.org/multipage/#update-the-source-set>
    pub fn update_source_set(&mut self, el: &Element) {
        // Step 1. Set el's source set to an empty source set.
        *self = SourceSet::new();

        // Step 2. Let elements be « el ».
        // Step 3. If el is an img element whose parent node is a picture element, then replace the
        // contents of elements with el's parent node's child elements, retaining relative order.
        // Step 4. Let img be el if el is an img element, otherwise null.
        let img = el.downcast::<HTMLImageElement>();
        let parent = el.upcast::<Node>().GetParentElement();
        let elements = match parent.as_ref() {
            Some(p) => {
                if p.is::<HTMLPictureElement>() {
                    p.upcast::<Node>()
                        .children()
                        .filter_map(DomRoot::downcast::<Element>)
                        .map(|n| DomRoot::from_ref(&*n))
                        .collect()
                } else {
                    vec![DomRoot::from_ref(el)]
                }
            },
            None => vec![DomRoot::from_ref(el)],
        };

        // Step 5. For each child in elements:
        for child in &elements {
            // Step 5.1. If child is el:
            if *child == DomRoot::from_ref(el) {
                let (default_source, srcset, sizes) = if el.is::<HTMLImageElement>() {
                    // Step 5.1.4: If el is an img element that has a srcset attribute, then
                    // set srcset to that attribute's value.
                    let srcset = el
                        .get_attribute_string_value(&local_name!("srcset"))
                        .unwrap_or_default();
                    // Step 5.1.6: If el is an img element that has a sizes attribute, then set sizes to that attribute's value.
                    let sizes = el
                        .get_attribute_string_value(&local_name!("sizes"))
                        .unwrap_or_default();
                    // Step 5.1.8: If el is an img element that has a src attribute, then set default source to that attribute's value.
                    let default_source = el
                        .get_attribute_string_value(&local_name!("src"))
                        .unwrap_or_default();
                    (default_source, srcset, sizes)
                } else if el.is::<HTMLLinkElement>() {
                    // Step 5.1.5: Otherwise, if el is a link element that has an imagesrcset attribute, then set srcset to that attribute's value.
                    let srcset = el
                        .get_attribute_string_value(&local_name!("imagesrcset"))
                        .unwrap_or_default();
                    // Step 5.1.7: Otherwise, if el is a link element that has an imagesizes attribute, then set sizes to that attribute's value.
                    let sizes = el
                        .get_attribute_string_value(&local_name!("imagesizes"))
                        .unwrap_or_default();
                    // Step 5.1.9: Otherwise, if el is a link element that has an href attribute, then set default source to that attribute's value.
                    let default_source = el
                        .get_attribute_string_value(&local_name!("href"))
                        .unwrap_or_default();
                    (default_source, srcset, sizes)
                } else {
                    // Step 5.1.1: Let default source be the empty string.
                    // Step 5.1.2: Let srcset be the empty string.
                    // Step 5.1.3: Let sizes be the empty string.
                    (String::new(), String::new(), String::new())
                };

                // Step 5.1.10. Set el's source set to the result of creating a source set given
                // default source, srcset, sizes, and img.
                *self = SourceSet::create_source_set(
                    &default_source,
                    &srcset,
                    &sizes,
                    &el.owner_document(),
                );

                // Step 5.1.11. Return.
                return;
            }
            // Spec note: If el is a link element, then elements contains only el, so this step
            // will be reached immediately and the rest of the algorithm will not run.
            debug_assert!(!el.is::<HTMLLinkElement>());
            // Step 5.2. If child is not a source element, then continue.
            if !child.is::<HTMLSourceElement>() {
                continue;
            }

            let mut source_set = SourceSet::new();

            // Step 5.3. If child does not have a srcset attribute, continue to the next child.
            // Step 5.4. Parse child's srcset attribute and let source set be the returned source
            // set.
            match child.get_attribute_string_value(&local_name!("srcset")) {
                Some(srcset) => {
                    source_set.image_sources = parse_a_srcset_attribute(&srcset);
                },
                _ => continue,
            }

            // Step 5.5. If source set has zero image sources, continue to the next child.
            if source_set.image_sources.is_empty() {
                continue;
            }

            // Step 5.6. If child has a media attribute, and its value does not match the
            // environment, continue to the next child.
            if let Some(media) = child.get_attribute_string_value(&local_name!("media")) &&
                !MediaList::matches_environment(&child.owner_document(), &media)
            {
                continue;
            }

            // Step 5.7. Parse child's sizes attribute with img, and let source set's source size be
            // the returned value.
            if let Some(sizes) = child.get_attribute_string_value(&local_name!("sizes")) {
                source_set.source_size = parse_a_sizes_attribute(&sizes);
            }

            // Step 5.8. If child has a type attribute, and its value is an unknown or unsupported
            // MIME type, continue to the next child.
            if let Some(type_) = child.get_attribute_string_value(&local_name!("type")) &&
                !is_supported_image_mime_type(&type_)
            {
                continue;
            }

            // Step 5.9. If child has width or height attributes, set el's dimension attribute
            // source to child. Otherwise, set el's dimension attribute source to el.
            if let Some(image) = img {
                if child.has_attribute(&local_name!("width")) ||
                    child.has_attribute(&local_name!("height"))
                {
                    image.set_dimension_attribute_source(Some(child));
                } else {
                    image.set_dimension_attribute_source(Some(el));
                }
            }

            // Step 5.10. Normalize the source densities of source set.
            source_set.normalise_source_densities(&el.owner_document());

            // Step 5.11. Set el's source set to source set.
            *self = source_set;

            // Step 5.12. Return.
            return;
        }
    }

    pub fn evaluate_source_size_list(&self, document: &Document) -> Au {
        let quirks_mode = document.quirks_mode();
        self.source_size
            .evaluate(document.window().layout().device(), quirks_mode)
    }

    /// <https://html.spec.whatwg.org/multipage/#normalise-the-source-densities>
    pub fn normalise_source_densities(&mut self, document: &Document) {
        // Step 1. Let source size be source set's source size.
        let source_size = self.evaluate_source_size_list(document);

        // Step 2. For each image source in source set:
        for image_source in self.image_sources.iter_mut() {
            // Step 2.1. If the image source has a pixel density descriptor, continue to the next
            // image source.
            if image_source.descriptor.density.is_some() {
                continue;
            }

            // Step 2.2. Otherwise, if the image source has a width descriptor, replace the width
            // descriptor with a pixel density descriptor with a value of the width descriptor value
            // divided by source size and a unit of x.
            if let Some(width) = image_source.descriptor.width {
                image_source.descriptor.density = Some(width as f64 / source_size.to_f64_px());
            } else {
                // Step 2.3. Otherwise, give the image source a pixel density descriptor of 1x.
                image_source.descriptor.density = Some(1_f64);
            }
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#select-an-image-source>
    pub fn select_image_source(&mut self, element: &Element) -> Option<(USVString, f64)> {
        // Step 1. Update the source set for el.
        self.update_source_set(element);

        // Step 2. If el's source set is empty, return null as the URL and undefined as the pixel
        // density.
        if self.image_sources.is_empty() {
            return None;
        }

        // Step 3. Return the result of selecting an image from el's source set.
        self.select_image_source_from_source_set(&element.owner_document())
    }

    /// <https://html.spec.whatwg.org/multipage/#select-an-image-source-from-a-source-set>
    pub fn select_image_source_from_source_set(
        &self,
        document: &Document,
    ) -> Option<(USVString, f64)> {
        // Step 1. If an entry b in sourceSet has the same associated pixel density descriptor as an
        // earlier entry a in sourceSet, then remove entry b. Repeat this step until none of the
        // entries in sourceSet have the same associated pixel density descriptor as an earlier
        // entry.
        let len = self.image_sources.len();

        // Using FxHash is ok here as the indices are just 0..len
        let mut repeat_indices = FxHashSet::default();
        for outer_index in 0..len {
            if repeat_indices.contains(&outer_index) {
                continue;
            }
            let imgsource = &self.image_sources[outer_index];
            let pixel_density = imgsource.descriptor.density.unwrap();
            for inner_index in (outer_index + 1)..len {
                let imgsource2 = &self.image_sources[inner_index];
                if pixel_density == imgsource2.descriptor.density.unwrap() {
                    repeat_indices.insert(inner_index);
                }
            }
        }

        let mut max = (0f64, 0);
        let img_sources = &mut vec![];
        for (index, image_source) in self.image_sources.iter().enumerate() {
            if repeat_indices.contains(&index) {
                continue;
            }
            let den = image_source.descriptor.density.unwrap();
            if max.0 < den {
                max = (den, img_sources.len());
            }
            img_sources.push(image_source);
        }

        // Step 2. In an implementation-defined manner, choose one image source from sourceSet. Let
        // selectedSource be this choice.
        let mut best_candidate = max;
        let device_pixel_ratio = document
            .window()
            .viewport_details()
            .hidpi_scale_factor
            .get() as f64;
        for (index, image_source) in img_sources.iter().enumerate() {
            let current_den = image_source.descriptor.density.unwrap();
            if current_den < best_candidate.0 && current_den >= device_pixel_ratio {
                best_candidate = (current_den, index);
            }
        }
        let selected_source = img_sources.remove(best_candidate.1).clone();

        // Step 3. Return selectedSource and its associated pixel density.
        Some((
            USVString(selected_source.url),
            selected_source.descriptor.density.unwrap(),
        ))
    }
}

/// <https://html.spec.whatwg.org/multipage/#parse-a-sizes-attribute>
pub fn parse_a_sizes_attribute(value: &str) -> SourceSizeList {
    let mut input = ParserInput::new(value);
    let mut parser = Parser::new(&mut input);
    // FIXME(emilio): why ::empty() instead of ::DEFAULT? Also, what do
    // browsers do regarding quirks-mode in a media list?
    let context = parser_context_for_anonymous_content(
        CssRuleType::Style,
        ParsingMode::empty(),
        &ANONYMOUS_CONTENT_URL_DATA,
    );
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

/// Returns true if the given image MIME type is supported.
fn is_supported_image_mime_type(input: &str) -> bool {
    // Remove any leading and trailing HTTP whitespace from input.
    let mime_type = input.trim();

    // <https://mimesniff.spec.whatwg.org/#mime-type-essence>
    let mime_type_essence = match mime_type.find(';') {
        Some(semi) => &mime_type[..semi],
        _ => mime_type,
    };

    // The HTML specification says the type attribute may be present and if present, the value
    // must be a valid MIME type string. However an empty type attribute is implicitly supported
    // to match the behavior of other browsers.
    // <https://html.spec.whatwg.org/multipage/#attr-source-type>
    if mime_type_essence.is_empty() {
        return true;
    }

    SUPPORTED_IMAGE_MIME_TYPES.contains(&mime_type_essence)
}
