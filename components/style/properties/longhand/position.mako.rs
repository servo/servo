/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

<%! from data import to_rust_ident %>
<%namespace name="helpers" file="/helpers.mako.rs" />
<% from data import ALL_SIZES, PHYSICAL_SIDES, LOGICAL_SIDES %>

<% data.new_style_struct("Position", inherited=False) %>

// "top" / "left" / "bottom" / "right"
% for side in PHYSICAL_SIDES:
    ${helpers.predefined_type(side, "LengthOrPercentageOrAuto",
                              "computed::LengthOrPercentageOrAuto::Auto",
                              spec="https://www.w3.org/TR/CSS2/visuren.html#propdef-%s" % side,
                              animation_value_type="ComputedValue",
                              allow_quirks=True)}
% endfor
// offset-* logical properties, map to "top" / "left" / "bottom" / "right"
% for side in LOGICAL_SIDES:
    ${helpers.predefined_type("offset-%s" % side, "LengthOrPercentageOrAuto",
                              "computed::LengthOrPercentageOrAuto::Auto",
                              spec="https://drafts.csswg.org/css-logical-props/#propdef-offset-%s" % side,
                              animation_value_type="ComputedValue", logical=True)}
% endfor

${helpers.predefined_type("z-index", "IntegerOrAuto",
                          "Either::Second(Auto)",
                          spec="https://www.w3.org/TR/CSS2/visuren.html#z-index",
                          flags="CREATES_STACKING_CONTEXT",
                          animation_value_type="ComputedValue")}


// CSS Flexible Box Layout Module Level 1
// http://www.w3.org/TR/css3-flexbox/

// Flex container properties
${helpers.single_keyword("flex-direction", "row row-reverse column column-reverse",
                         spec="https://drafts.csswg.org/css-flexbox/#flex-direction-property",
                         extra_prefixes="webkit", animation_value_type="none")}

${helpers.single_keyword("flex-wrap", "nowrap wrap wrap-reverse",
                         spec="https://drafts.csswg.org/css-flexbox/#flex-wrap-property",
                         extra_prefixes="webkit", animation_value_type="none")}

% if product == "servo":
    // FIXME: Update Servo to support the same Syntax as Gecko.
    ${helpers.single_keyword("justify-content", "flex-start stretch flex-end center space-between space-around",
                             extra_prefixes="webkit",
                             spec="https://drafts.csswg.org/css-align/#propdef-justify-content",
                             animation_value_type="none")}
% else:
    ${helpers.predefined_type(name="justify-content",
                              type="AlignJustifyContent",
                              initial_value="specified::AlignJustifyContent::normal()",
                              spec="https://drafts.csswg.org/css-align/#propdef-justify-content",
                              extra_prefixes="webkit",
                              animation_value_type="none")}
% endif

% if product == "servo":
    // FIXME: Update Servo to support the same Syntax as Gecko.
    ${helpers.single_keyword("align-content", "stretch flex-start flex-end center space-between space-around",
                             extra_prefixes="webkit",
                             spec="https://drafts.csswg.org/css-align/#propdef-align-content",
                             animation_value_type="none")}

    ${helpers.single_keyword("align-items",
                             "stretch flex-start flex-end center baseline",
                             extra_prefixes="webkit",
                             spec="https://drafts.csswg.org/css-flexbox/#align-items-property",
                             animation_value_type="discrete")}
% else:
    ${helpers.predefined_type(name="align-content",
                              type="AlignJustifyContent",
                              initial_value="specified::AlignJustifyContent::normal()",
                              spec="https://drafts.csswg.org/css-align/#propdef-align-content",
                              extra_prefixes="webkit",
                              animation_value_type="none")}

    ${helpers.predefined_type(name="align-items",
                              type="AlignItems",
                              initial_value="specified::AlignItems::normal()",
                              spec="https://drafts.csswg.org/css-align/#propdef-align-items",
                              extra_prefixes="webkit",
                              animation_value_type="discrete")}

    ${helpers.predefined_type(name="justify-items",
                              type="JustifyItems",
                              initial_value="specified::JustifyItems::auto()",
                              spec="https://drafts.csswg.org/css-align/#propdef-justify-items",
                              animation_value_type="none")}
% endif

// Flex item properties
${helpers.predefined_type("flex-grow", "Number",
                          "0.0", "parse_non_negative",
                          spec="https://drafts.csswg.org/css-flexbox/#flex-grow-property",
                          extra_prefixes="webkit",
                          animation_value_type="ComputedValue")}

${helpers.predefined_type("flex-shrink", "Number",
                          "1.0", "parse_non_negative",
                          spec="https://drafts.csswg.org/css-flexbox/#flex-shrink-property",
                          extra_prefixes="webkit",
                          animation_value_type="ComputedValue")}

// https://drafts.csswg.org/css-align/#align-self-property
% if product == "servo":
    // FIXME: Update Servo to support the same syntax as Gecko.
    ${helpers.single_keyword("align-self", "auto stretch flex-start flex-end center baseline",
                             need_clone=True,
                             extra_prefixes="webkit",
                             spec="https://drafts.csswg.org/css-flexbox/#propdef-align-self",
                             animation_value_type="none")}
% else:
    ${helpers.predefined_type(name="align-self",
                              type="AlignJustifySelf",
                              initial_value="specified::AlignJustifySelf::auto()",
                              spec="https://drafts.csswg.org/css-align/#align-self-property",
                              extra_prefixes="webkit",
                              animation_value_type="none")}

    ${helpers.predefined_type(name="justify-self",
                              type="AlignJustifySelf",
                              initial_value="specified::AlignJustifySelf::auto()",
                              spec="https://drafts.csswg.org/css-align/#justify-self-property",
                              animation_value_type="none")}
% endif

// https://drafts.csswg.org/css-flexbox/#propdef-order
${helpers.predefined_type("order", "Integer", "0",
                          extra_prefixes="webkit",
                          animation_value_type="ComputedValue",
                          spec="https://drafts.csswg.org/css-flexbox/#order-property")}

% if product == "gecko":
    // FIXME: Gecko doesn't support content value yet.
    ${helpers.gecko_size_type("flex-basis", "MozLength", "auto()",
                              logical=False,
                              spec="https://drafts.csswg.org/css-flexbox/#flex-basis-property",
                              extra_prefixes="webkit",
                              animation_value_type="ComputedValue")}
% else:
    // FIXME: This property should be animatable.
    ${helpers.predefined_type("flex-basis",
                              "LengthOrPercentageOrAutoOrContent",
                              "computed::LengthOrPercentageOrAutoOrContent::Auto",
                              "parse_non_negative",
                              spec="https://drafts.csswg.org/css-flexbox/#flex-basis-property",
                              extra_prefixes="webkit",
                              animation_value_type="none")}
% endif
% for (size, logical) in ALL_SIZES:
    <%
      spec = "https://drafts.csswg.org/css-box/#propdef-%s"
      if logical:
        spec = "https://drafts.csswg.org/css-logical-props/#propdef-%s"
    %>
    % if product == "gecko":
        // width, height, block-size, inline-size
        ${helpers.gecko_size_type("%s" % size, "MozLength", "auto()",
                                  logical,
                                  spec=spec % size,
                                  animation_value_type="ComputedValue")}
        // min-width, min-height, min-block-size, min-inline-size,
        // max-width, max-height, max-block-size, max-inline-size
        ${helpers.gecko_size_type("min-%s" % size, "MozLength", "auto()",
                                  logical,
                                  spec=spec % size,
                                  animation_value_type="ComputedValue")}
        ${helpers.gecko_size_type("max-%s" % size, "MaxLength", "none()",
                                  logical,
                                  spec=spec % size,
                                  animation_value_type="ComputedValue")}
    % else:
        // servo versions (no keyword support)
        ${helpers.predefined_type("%s" % size,
                                  "LengthOrPercentageOrAuto",
                                  "computed::LengthOrPercentageOrAuto::Auto",
                                  "parse_non_negative",
                                  spec=spec % size,
                                  allow_quirks=not logical,
                                  animation_value_type="ComputedValue", logical = logical)}
        ${helpers.predefined_type("min-%s" % size,
                                  "LengthOrPercentage",
                                  "computed::LengthOrPercentage::Length(Au(0))",
                                  "parse_non_negative",
                                  spec=spec % ("min-%s" % size),
                                  animation_value_type="ComputedValue",
                                  logical=logical,
                                  allow_quirks=not logical)}
        ${helpers.predefined_type("max-%s" % size,
                                  "LengthOrPercentageOrNone",
                                  "computed::LengthOrPercentageOrNone::None",
                                  "parse_non_negative",
                                  spec=spec % ("min-%s" % size),
                                  animation_value_type="ComputedValue",
                                  logical=logical,
                                  allow_quirks=not logical)}
    % endif
% endfor

${helpers.single_keyword("box-sizing",
                         "content-box border-box",
                         extra_prefixes="moz webkit",
                         spec="https://drafts.csswg.org/css-ui/#propdef-box-sizing",
                         animation_value_type="none")}

${helpers.single_keyword("object-fit", "fill contain cover none scale-down",
                         products="gecko", animation_value_type="none",
                         spec="https://drafts.csswg.org/css-images/#propdef-object-fit")}

${helpers.predefined_type("object-position",
                          "Position",
                          "computed::Position::zero()",
                          products="gecko",
                          boxed="True",
                          spec="https://drafts.csswg.org/css-images-3/#the-object-position",
                          animation_value_type="ComputedValue")}

% for kind in ["row", "column"]:
    ${helpers.predefined_type("grid-%s-gap" % kind,
                              "LengthOrPercentage",
                              "computed::LengthOrPercentage::Length(Au(0))",
                              "parse_non_negative",
                              spec="https://drafts.csswg.org/css-grid/#propdef-grid-%s-gap" % kind,
                              animation_value_type="ComputedValue",
                              products="gecko")}

    % for range in ["start", "end"]:
        ${helpers.predefined_type("grid-%s-%s" % (kind, range),
                                  "GridLine",
                                  "Default::default()",
                                  animation_value_type="none",
                                  spec="https://drafts.csswg.org/css-grid/#propdef-grid-%s-%s" % (kind, range),
                                  products="gecko",
                                  boxed=True)}
    % endfor

    // NOTE: According to the spec, this should handle multiple values of `<track-size>`,
    // but gecko supports only a single value
    ${helpers.predefined_type("grid-auto-%ss" % kind,
                              "TrackSize",
                              "Default::default()",
                              animation_value_type="none",
                              spec="https://drafts.csswg.org/css-grid/#propdef-grid-auto-%ss" % kind,
                              products="gecko",
                              boxed=True)}

    // NOTE: The spec lists only `none | <track-list> | <auto-track-list>`, but gecko seems to support
    // `subgrid <line-name-list>?` in addition to this (probably old spec). We should support it soon.
    ${helpers.predefined_type("grid-template-%ss" % kind,
                              "TrackListOrNone",
                              "Either::Second(None_)",
                              products="gecko",
                              spec="https://drafts.csswg.org/css-grid/#propdef-grid-template-%ss" % kind,
                              boxed=True,
                              animation_value_type="none")}

% endfor

<%helpers:longhand name="grid-auto-flow"
        spec="https://drafts.csswg.org/css-grid/#propdef-grid-auto-flow"
        products="gecko"
        animation_value_type="none">
    use std::fmt;
    use style_traits::ToCss;
    use values::computed::ComputedValueAsSpecified;

    pub type SpecifiedValue = computed_value::T;

    pub mod computed_value {
        #[derive(PartialEq, Clone, Eq, Copy, Debug)]
        #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
        pub enum AutoFlow {
            Row,
            Column,
        }

        #[derive(PartialEq, Clone, Eq, Copy, Debug)]
        #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
        pub struct T {
            pub autoflow: AutoFlow,
            pub dense: bool,
        }
    }

    no_viewport_percentage!(SpecifiedValue);
    impl ComputedValueAsSpecified for SpecifiedValue {}

    impl ToCss for computed_value::T {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            dest.write_str(match self.autoflow {
                computed_value::AutoFlow::Column => "column",
                computed_value::AutoFlow::Row => "row"
            })?;

            if self.dense { dest.write_str(" dense")?; }
            Ok(())
        }
    }

    #[inline]
    pub fn get_initial_value() -> computed_value::T {
        computed_value::T {
            autoflow: computed_value::AutoFlow::Row,
            dense: false
        }
    }

    /// [ row | column ] || dense
    pub fn parse(_context: &ParserContext, input: &mut Parser) -> Result<SpecifiedValue, ()> {
        use self::computed_value::AutoFlow;

        let mut value = None;
        let mut dense = false;

        while !input.is_exhausted() {
            match_ignore_ascii_case! { &input.expect_ident()?,
                "row" if value.is_none() => {
                    value = Some(AutoFlow::Row);
                    continue
                },
                "column" if value.is_none() => {
                    value = Some(AutoFlow::Column);
                    continue
                },
                "dense" if !dense => {
                    dense = true;
                    continue
                },
                _ => return Err(())
            }
        }

        if value.is_some() || dense {
            Ok(computed_value::T {
                autoflow: value.unwrap_or(AutoFlow::Row),
                dense: dense,
            })
        } else {
            Err(())
        }
    }
</%helpers:longhand>

<%helpers:longhand name="grid-template-areas"
        spec="https://drafts.csswg.org/css-grid/#propdef-grid-template-areas"
        products="gecko"
        animation_value_type="none"
        disable_when_testing="True"
        boxed="True">
    use cssparser::serialize_string;
    use std::collections::HashMap;
    use std::fmt;
    use std::ops::Range;
    use str::HTML_SPACE_CHARACTERS;
    use style_traits::ToCss;
    use values::computed::ComputedValueAsSpecified;

    pub mod computed_value {
        pub use super::SpecifiedValue as T;
    }

    pub type SpecifiedValue = Either<TemplateAreas, None_>;

    #[inline]
    pub fn get_initial_value() -> computed_value::T {
        Either::Second(None_)
    }

    pub fn parse(context: &ParserContext, input: &mut Parser) -> Result<SpecifiedValue, ()> {
        SpecifiedValue::parse(context, input)
    }

    #[derive(Clone, PartialEq)]
    pub struct TemplateAreas {
        pub areas: Box<[NamedArea]>,
        pub strings: Box<[Box<str>]>,
        pub width: u32,
    }

    #[derive(Clone, PartialEq)]
    pub struct NamedArea {
        pub name: Box<str>,
        pub rows: Range<u32>,
        pub columns: Range<u32>,
    }

    no_viewport_percentage!(TemplateAreas);
    impl ComputedValueAsSpecified for TemplateAreas {}

    impl Parse for TemplateAreas {
        fn parse(_context: &ParserContext, input: &mut Parser) -> Result<Self, ()> {
            let mut strings = vec![];
            while let Ok(string) = input.try(Parser::expect_string) {
                strings.push(string.into_owned().into_boxed_str());
            }

            TemplateAreas::from_vec(strings)
        }
    }

    impl TemplateAreas {
        pub fn from_vec(strings: Vec<Box<str>>) -> Result<TemplateAreas, ()> {
            if strings.is_empty() {
                return Err(());
            }
            let mut areas: Vec<NamedArea> = vec![];
            let mut width = 0;
            {
                let mut row = 0u32;
                let mut area_indices = HashMap::<(&str), usize>::new();
                for string in &strings {
                    let mut current_area_index: Option<usize> = None;
                    row += 1;
                    let mut column = 0u32;
                    for token in Tokenizer(string) {
                        column += 1;
                        let token = if let Some(token) = token? {
                            token
                        } else {
                            if let Some(index) = current_area_index.take() {
                                if areas[index].columns.end != column {
                                    return Err(());
                                }
                            }
                            continue;
                        };
                        if let Some(index) = current_area_index {
                            if &*areas[index].name == token {
                                if areas[index].rows.start == row {
                                    areas[index].columns.end += 1;
                                }
                                continue;
                            }
                            if areas[index].columns.end != column {
                                return Err(());
                            }
                        }
                        if let Some(index) = area_indices.get(token).cloned() {
                            if areas[index].columns.start != column || areas[index].rows.end != row {
                                return Err(());
                            }
                            areas[index].rows.end += 1;
                            current_area_index = Some(index);
                            continue;
                        }
                        let index = areas.len();
                        areas.push(NamedArea {
                            name: token.to_owned().into_boxed_str(),
                            columns: column..(column + 1),
                            rows: row..(row + 1),
                        });
                        assert!(area_indices.insert(token, index).is_none());
                        current_area_index = Some(index);
                    }
                    if let Some(index) = current_area_index {
                        if areas[index].columns.end != column + 1 {
                            assert!(areas[index].rows.start != row);
                            return Err(());
                        }
                    }
                    if row == 1 {
                        width = column;
                    } else if width != column {
                        return Err(());
                    }
                }
            }
            Ok(TemplateAreas {
                areas: areas.into_boxed_slice(),
                strings: strings.into_boxed_slice(),
                width: width,
            })
        }
    }

    impl ToCss for TemplateAreas {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            for (i, string) in self.strings.iter().enumerate() {
                if i != 0 {
                    dest.write_str(" ")?;
                }
                serialize_string(string, dest)?;
            }
            Ok(())
        }
    }

    struct Tokenizer<'a>(&'a str);

    impl<'a> Iterator for Tokenizer<'a> {
        type Item = Result<Option<(&'a str)>, ()>;

        fn next(&mut self) -> Option<Self::Item> {
            let rest = self.0.trim_left_matches(HTML_SPACE_CHARACTERS);
            if rest.is_empty() {
                return None;
            }
            if rest.starts_with('.') {
                self.0 = &rest[rest.find(|c| c != '.').unwrap_or(rest.len())..];
                return Some(Ok(None));
            }
            if !rest.starts_with(is_name_code_point) {
                return Some(Err(()));
            }
            let token_len = rest.find(|c| !is_name_code_point(c)).unwrap_or(rest.len());
            let token = &rest[..token_len];
            self.0 = &rest[token_len..];
            Some(Ok(Some(token)))
        }
    }

    fn is_name_code_point(c: char) -> bool {
        c >= 'A' && c <= 'Z' || c >= 'a' && c <= 'z' ||
        c >= '\u{80}' || c == '_' ||
        c >= '0' && c <= '9' || c == '-'
    }
</%helpers:longhand>
