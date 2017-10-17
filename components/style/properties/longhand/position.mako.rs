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

#[cfg(feature = "gecko")]
macro_rules! impl_align_conversions {
    ($name: path) => {
        impl From<u8> for $name {
            fn from(bits: u8) -> $name {
                $name(::values::specified::align::AlignFlags::from_bits(bits)
                      .expect("bits contain valid flag"))
            }
        }

        impl From<$name> for u8 {
            fn from(v: $name) -> u8 {
                v.0.bits()
            }
        }
    };
}

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
                         extra_prefixes="webkit", animation_value_type="discrete")}

${helpers.single_keyword("flex-wrap", "nowrap wrap wrap-reverse",
                         spec="https://drafts.csswg.org/css-flexbox/#flex-wrap-property",
                         extra_prefixes="webkit", animation_value_type="discrete")}

% if product == "servo":
    // FIXME: Update Servo to support the same Syntax as Gecko.
    ${helpers.single_keyword("justify-content", "flex-start stretch flex-end center space-between space-around",
                             extra_prefixes="webkit",
                             spec="https://drafts.csswg.org/css-align/#propdef-justify-content",
                             animation_value_type="discrete")}
% else:
    ${helpers.predefined_type(name="justify-content",
                              type="AlignJustifyContent",
                              initial_value="specified::AlignJustifyContent::normal()",
                              spec="https://drafts.csswg.org/css-align/#propdef-justify-content",
                              extra_prefixes="webkit",
                              animation_value_type="discrete")}
% endif

% if product == "servo":
    // FIXME: Update Servo to support the same Syntax as Gecko.
    ${helpers.single_keyword("align-content", "stretch flex-start flex-end center space-between space-around",
                             extra_prefixes="webkit",
                             spec="https://drafts.csswg.org/css-align/#propdef-align-content",
                             animation_value_type="discrete")}

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
                              animation_value_type="discrete")}

    ${helpers.predefined_type(name="align-items",
                              type="AlignItems",
                              initial_value="specified::AlignItems::normal()",
                              spec="https://drafts.csswg.org/css-align/#propdef-align-items",
                              extra_prefixes="webkit",
                              animation_value_type="discrete")}

    #[cfg(feature = "gecko")]
    impl_align_conversions!(::values::specified::align::AlignItems);

    ${helpers.predefined_type(name="justify-items",
                              type="JustifyItems",
                              initial_value="computed::JustifyItems::auto()",
                              spec="https://drafts.csswg.org/css-align/#propdef-justify-items",
                              animation_value_type="discrete")}

    #[cfg(feature = "gecko")]
    impl_align_conversions!(::values::specified::align::JustifyItems);
% endif

// Flex item properties
${helpers.predefined_type("flex-grow", "NonNegativeNumber",
                          "From::from(0.0)",
                          spec="https://drafts.csswg.org/css-flexbox/#flex-grow-property",
                          extra_prefixes="webkit",
                          animation_value_type="NonNegativeNumber")}

${helpers.predefined_type("flex-shrink", "NonNegativeNumber",
                          "From::from(1.0)",
                          spec="https://drafts.csswg.org/css-flexbox/#flex-shrink-property",
                          extra_prefixes="webkit",
                          animation_value_type="NonNegativeNumber")}

// https://drafts.csswg.org/css-align/#align-self-property
% if product == "servo":
    // FIXME: Update Servo to support the same syntax as Gecko.
    ${helpers.single_keyword("align-self", "auto stretch flex-start flex-end center baseline",
                             extra_prefixes="webkit",
                             spec="https://drafts.csswg.org/css-flexbox/#propdef-align-self",
                             animation_value_type="discrete")}
% else:
    ${helpers.predefined_type(name="align-self",
                              type="AlignJustifySelf",
                              initial_value="specified::AlignJustifySelf::auto()",
                              spec="https://drafts.csswg.org/css-align/#align-self-property",
                              extra_prefixes="webkit",
                              animation_value_type="discrete")}

    ${helpers.predefined_type(name="justify-self",
                              type="AlignJustifySelf",
                              initial_value="specified::AlignJustifySelf::auto()",
                              spec="https://drafts.csswg.org/css-align/#justify-self-property",
                              animation_value_type="discrete")}

    #[cfg(feature = "gecko")]
    impl_align_conversions!(::values::specified::align::AlignJustifySelf);
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
                              animation_value_type="MozLength")}
% else:
    // FIXME: This property should be animatable.
    ${helpers.predefined_type("flex-basis",
                              "FlexBasis",
                              "computed::FlexBasis::auto()",
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
                                  animation_value_type="MozLength")}
        // min-width, min-height, min-block-size, min-inline-size,
        // max-width, max-height, max-block-size, max-inline-size
        ${helpers.gecko_size_type("min-%s" % size, "MozLength", "auto()",
                                  logical,
                                  spec=spec % size,
                                  animation_value_type="MozLength")}
        ${helpers.gecko_size_type("max-%s" % size, "MaxLength", "none()",
                                  logical,
                                  spec=spec % size,
                                  animation_value_type="MaxLength")}
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
                                  "computed::LengthOrPercentage::Length(computed::Length::new(0.))",
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
                         gecko_enum_prefix="StyleBoxSizing",
                         custom_consts={ "content-box": "Content", "border-box": "Border" },
                         animation_value_type="discrete")}

${helpers.single_keyword("object-fit", "fill contain cover none scale-down",
                         products="gecko", animation_value_type="discrete",
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
                              "NonNegativeLengthOrPercentage",
                              "computed::NonNegativeLengthOrPercentage::zero()",
                              spec="https://drafts.csswg.org/css-grid/#propdef-grid-%s-gap" % kind,
                              animation_value_type="NonNegativeLengthOrPercentage",
                              products="gecko")}

    % for range in ["start", "end"]:
        ${helpers.predefined_type("grid-%s-%s" % (kind, range),
                                  "GridLine",
                                  "Default::default()",
                                  animation_value_type="discrete",
                                  spec="https://drafts.csswg.org/css-grid/#propdef-grid-%s-%s" % (kind, range),
                                  products="gecko",
                                  boxed=True)}
    % endfor

    // NOTE: According to the spec, this should handle multiple values of `<track-size>`,
    // but gecko supports only a single value
    ${helpers.predefined_type("grid-auto-%ss" % kind,
                              "TrackSize",
                              "Default::default()",
                              animation_value_type="discrete",
                              spec="https://drafts.csswg.org/css-grid/#propdef-grid-auto-%ss" % kind,
                              products="gecko",
                              boxed=True)}

    ${helpers.predefined_type("grid-template-%ss" % kind,
                              "GridTemplateComponent",
                              "specified::GenericGridTemplateComponent::None",
                              products="gecko",
                              spec="https://drafts.csswg.org/css-grid/#propdef-grid-template-%ss" % kind,
                              boxed=True,
                              animation_value_type="discrete")}

% endfor

<%helpers:longhand name="grid-auto-flow"
        spec="https://drafts.csswg.org/css-grid/#propdef-grid-auto-flow"
        products="gecko"
        animation_value_type="discrete">
    use std::fmt;
    use style_traits::ToCss;

    pub type SpecifiedValue = computed_value::T;

    pub mod computed_value {
        #[derive(Clone, Copy, Debug, Eq, MallocSizeOf, PartialEq, ToComputedValue)]
        pub enum AutoFlow {
            Row,
            Column,
        }

        #[derive(Clone, Copy, Debug, Eq, MallocSizeOf, PartialEq, ToComputedValue)]
        pub struct T {
            pub autoflow: AutoFlow,
            pub dense: bool,
        }
    }

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
            dense: false,
        }
    }

    /// [ row | column ] || dense
    pub fn parse<'i, 't>(_context: &ParserContext, input: &mut Parser<'i, 't>)
                         -> Result<SpecifiedValue, ParseError<'i>> {
        use self::computed_value::AutoFlow;

        let mut value = None;
        let mut dense = false;

        while !input.is_exhausted() {
            let location = input.current_source_location();
            let ident = input.expect_ident()?;
            let success = match_ignore_ascii_case! { &ident,
                "row" if value.is_none() => {
                    value = Some(AutoFlow::Row);
                    true
                },
                "column" if value.is_none() => {
                    value = Some(AutoFlow::Column);
                    true
                },
                "dense" if !dense => {
                    dense = true;
                    true
                },
                _ => false
            };
            if !success {
                return Err(location.new_custom_error(SelectorParseErrorKind::UnexpectedIdent(ident.clone())));
            }
        }

        if value.is_some() || dense {
            Ok(computed_value::T {
                autoflow: value.unwrap_or(AutoFlow::Row),
                dense: dense,
            })
        } else {
            Err(input.new_custom_error(StyleParseErrorKind::UnspecifiedError))
        }
    }

    #[cfg(feature = "gecko")]
    impl From<u8> for SpecifiedValue {
        fn from(bits: u8) -> SpecifiedValue {
            use gecko_bindings::structs;
            use self::computed_value::AutoFlow;

            SpecifiedValue {
                autoflow:
                    if bits & structs::NS_STYLE_GRID_AUTO_FLOW_ROW as u8 != 0 {
                        AutoFlow::Row
                    } else {
                        AutoFlow::Column
                    },
                dense:
                    bits & structs::NS_STYLE_GRID_AUTO_FLOW_DENSE as u8 != 0,
            }
        }
    }

    #[cfg(feature = "gecko")]
    impl From<SpecifiedValue> for u8 {
        fn from(v: SpecifiedValue) -> u8 {
            use gecko_bindings::structs;
            use self::computed_value::AutoFlow;

            let mut result: u8 = match v.autoflow {
                AutoFlow::Row => structs::NS_STYLE_GRID_AUTO_FLOW_ROW as u8,
                AutoFlow::Column => structs::NS_STYLE_GRID_AUTO_FLOW_COLUMN as u8,
            };

            if v.dense {
                result |= structs::NS_STYLE_GRID_AUTO_FLOW_DENSE as u8;
            }
            result
        }
    }
</%helpers:longhand>

<%helpers:longhand name="grid-template-areas"
        spec="https://drafts.csswg.org/css-grid/#propdef-grid-template-areas"
        products="gecko"
        animation_value_type="discrete"
        boxed="True">
    use hash::FnvHashMap;
    use std::fmt;
    use std::ops::Range;
    use str::HTML_SPACE_CHARACTERS;
    use style_traits::ToCss;

    pub mod computed_value {
        pub use super::SpecifiedValue as T;
    }

    pub type SpecifiedValue = Either<TemplateAreas, None_>;

    #[inline]
    pub fn get_initial_value() -> computed_value::T {
        Either::Second(None_)
    }

    pub fn parse<'i, 't>(context: &ParserContext, input: &mut Parser<'i, 't>)
                         -> Result<SpecifiedValue, ParseError<'i>> {
        SpecifiedValue::parse(context, input)
    }

    #[cfg_attr(feature = "gecko", derive(MallocSizeOf))]
    #[derive(Clone, Debug, PartialEq)]
    pub struct TemplateAreas {
        pub areas: Box<[NamedArea]>,
        pub strings: Box<[Box<str>]>,
        pub width: u32,
    }

    #[cfg_attr(feature = "gecko", derive(MallocSizeOf))]
    #[derive(Clone, Debug, PartialEq)]
    pub struct NamedArea {
        pub name: Box<str>,
        pub rows: Range<u32>,
        pub columns: Range<u32>,
    }

    trivial_to_computed_value!(TemplateAreas);

    impl Parse for TemplateAreas {
        fn parse<'i, 't>(
            _context: &ParserContext,
            input: &mut Parser<'i, 't>,
        ) -> Result<Self, ParseError<'i>> {
            let mut strings = vec![];
            while let Ok(string) = input.try(|i| i.expect_string().map(|s| s.as_ref().into())) {
                strings.push(string);
            }

            TemplateAreas::from_vec(strings)
                .map_err(|()| input.new_custom_error(StyleParseErrorKind::UnspecifiedError))
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
                let mut area_indices = FnvHashMap::<(&str), usize>::default();
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
                string.to_css(dest)?;
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
