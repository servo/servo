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
                              animatable=True)}
% endfor
// offset-* logical properties, map to "top" / "left" / "bottom" / "right"
% for side in LOGICAL_SIDES:
    ${helpers.predefined_type("offset-%s" % side, "LengthOrPercentageOrAuto",
                              "computed::LengthOrPercentageOrAuto::Auto",
                              spec="https://drafts.csswg.org/css-logical-props/#propdef-offset-%s" % side,
                              animatable=True, logical=True)}
% endfor

${helpers.predefined_type("z-index", "IntegerOrAuto",
                          "Either::Second(Auto)",
                          spec="https://www.w3.org/TR/CSS2/visuren.html#z-index",
                          creates_stacking_context=True,
                          animatable="True")}

// CSS Flexible Box Layout Module Level 1
// http://www.w3.org/TR/css3-flexbox/

// Flex container properties
${helpers.single_keyword("flex-direction", "row row-reverse column column-reverse",
                         spec="https://drafts.csswg.org/css-flexbox/#flex-direction-property",
                         extra_prefixes="webkit", animatable=False)}

${helpers.single_keyword("flex-wrap", "nowrap wrap wrap-reverse",
                         spec="https://drafts.csswg.org/css-flexbox/#flex-wrap-property",
                         extra_prefixes="webkit", animatable=False)}

% if product == "servo":
    // FIXME: Update Servo to support the same Syntax as Gecko.
    ${helpers.single_keyword("justify-content", "flex-start stretch flex-end center space-between space-around",
                             extra_prefixes="webkit",
                             spec="https://drafts.csswg.org/css-align/#propdef-justify-content",
                             animatable=False)}
% else:
    ${helpers.predefined_type(name="justify-content",
                              type="AlignJustifyContent",
                              initial_value="specified::AlignJustifyContent::normal()",
                              spec="https://drafts.csswg.org/css-align/#propdef-justify-content",
                              extra_prefixes="webkit",
                              animatable=False)}
% endif

% if product == "servo":
    // FIXME: Update Servo to support the same Syntax as Gecko.
    ${helpers.single_keyword("align-content", "stretch flex-start flex-end center space-between space-around",
                             extra_prefixes="webkit",
                             spec="https://drafts.csswg.org/css-align/#propdef-align-content",
                             animatable=False)}

    ${helpers.single_keyword("align-items",
                             "stretch flex-start flex-end center baseline",
                             need_clone=True,
                             extra_prefixes="webkit",
                             spec="https://drafts.csswg.org/css-flexbox/#align-items-property",
                             animatable=False)}
% else:
    ${helpers.predefined_type(name="align-content",
                              type="AlignJustifyContent",
                              initial_value="specified::AlignJustifyContent::normal()",
                              spec="https://drafts.csswg.org/css-align/#propdef-align-content",
                              extra_prefixes="webkit",
                              animatable=False)}

    ${helpers.predefined_type(name="align-items",
                              type="AlignItems",
                              initial_value="specified::AlignItems::normal()",
                              spec="https://drafts.csswg.org/css-align/#propdef-align-items",
                              extra_prefixes="webkit",
                              animatable=False)}

    ${helpers.predefined_type(name="justify-items",
                              type="JustifyItems",
                              initial_value="specified::JustifyItems::auto()",
                              spec="https://drafts.csswg.org/css-align/#propdef-justify-items",
                              animatable=False)}
% endif

// Flex item properties
${helpers.predefined_type("flex-grow", "Number",
                          "0.0", "parse_non_negative",
                          spec="https://drafts.csswg.org/css-flexbox/#flex-grow-property",
                          extra_prefixes="webkit",
                          needs_context=False,
                          animatable=True)}

${helpers.predefined_type("flex-shrink", "Number",
                          "1.0", "parse_non_negative",
                          spec="https://drafts.csswg.org/css-flexbox/#flex-shrink-property",
                          extra_prefixes="webkit",
                          needs_context=False,
                          animatable=True)}

// https://drafts.csswg.org/css-align/#align-self-property
% if product == "servo":
    // FIXME: Update Servo to support the same syntax as Gecko.
    ${helpers.single_keyword("align-self", "auto stretch flex-start flex-end center baseline",
                             need_clone=True,
                             extra_prefixes="webkit",
                             spec="https://drafts.csswg.org/css-flexbox/#propdef-align-self",
                             animatable=False)}
% else:
    ${helpers.predefined_type(name="align-self",
                              type="AlignJustifySelf",
                              initial_value="specified::AlignJustifySelf::auto()",
                              spec="https://drafts.csswg.org/css-align/#align-self-property",
                              extra_prefixes="webkit",
                              animatable=False)}

    ${helpers.predefined_type(name="justify-self",
                              type="AlignJustifySelf",
                              initial_value="specified::AlignJustifySelf::auto()",
                              spec="https://drafts.csswg.org/css-align/#justify-self-property",
                              animatable=False)}
% endif

// https://drafts.csswg.org/css-flexbox/#propdef-order
${helpers.predefined_type("order", "Integer", "0",
                          extra_prefixes="webkit",
                          animatable=True,
                          spec="https://drafts.csswg.org/css-flexbox/#order-property")}

// FIXME: Gecko doesn't support content value yet.
// FIXME: This property should be animatable.
${helpers.predefined_type("flex-basis",
                          "LengthOrPercentageOrAuto" if product == "gecko" else
                          "LengthOrPercentageOrAutoOrContent",
                          "computed::LengthOrPercentageOrAuto::Auto" if product == "gecko" else
                          "computed::LengthOrPercentageOrAutoOrContent::Auto",
                          spec="https://drafts.csswg.org/css-flexbox/#flex-basis-property",
                          extra_prefixes="webkit",
                          animatable=True if product == "gecko" else False)}

% for (size, logical) in ALL_SIZES:
    <%
      spec = "https://drafts.csswg.org/css-box/#propdef-%s"
      if logical:
        spec = "https://drafts.csswg.org/css-logical-props/#propdef-%s"
    %>
    // width, height, block-size, inline-size
    ${helpers.predefined_type("%s" % size,
                              "LengthOrPercentageOrAuto",
                              "computed::LengthOrPercentageOrAuto::Auto",
                              "parse_non_negative",
                              needs_context=False,
                              spec=spec % size,
                              animatable=True, logical = logical)}
    % if product == "gecko":
        % for min_max in ["min", "max"]:
            <%
                MinMax = min_max.title()
                initial = "None" if "max" == min_max else "Auto"
            %>

            // min-width, min-height, min-block-size, min-inline-size,
            // max-width, max-height, max-block-size, max-inline-size
            //
            // Keyword values are only valid in the inline direction; they must
            // be replaced with auto/none in block.
            <%helpers:longhand name="${min_max}-${size}" spec="${spec % ('%s-%s' % (min_max, size))}"
                               animatable="True" logical="${logical}" predefined_type="${MinMax}Length">

                use std::fmt;
                use style_traits::ToCss;
                use values::HasViewportPercentage;
                use values::specified::${MinMax}Length;

                impl HasViewportPercentage for SpecifiedValue {
                    fn has_viewport_percentage(&self) -> bool {
                        self.0.has_viewport_percentage()
                    }
                }

                pub mod computed_value {
                    pub type T = ::values::computed::${MinMax}Length;
                }

                #[derive(PartialEq, Clone, Debug)]
                #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
                pub struct SpecifiedValue(${MinMax}Length);

                #[inline]
                pub fn get_initial_value() -> computed_value::T {
                    use values::computed::${MinMax}Length;
                    ${MinMax}Length::${initial}
                }
                fn parse(context: &ParserContext, input: &mut Parser) -> Result<SpecifiedValue, ()> {
                    ${MinMax}Length::parse(context, input).map(SpecifiedValue)
                }

                impl ToCss for SpecifiedValue {
                    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
                        self.0.to_css(dest)
                    }
                }

                impl ToComputedValue for SpecifiedValue {
                    type ComputedValue = computed_value::T;
                    #[inline]
                    fn to_computed_value(&self, context: &Context) -> computed_value::T {
                        use values::computed::${MinMax}Length;
                        let computed = self.0.to_computed_value(context);

                        // filter out keyword values in the block direction
                        % if logical:
                            % if "block" in size:
                                if let ${MinMax}Length::ExtremumLength(..) = computed {
                                    return get_initial_value()
                                }
                            % endif
                        % else:
                            if let ${MinMax}Length::ExtremumLength(..) = computed {
                                <% is_height = "true" if "height" in size else "false" %>
                                if ${is_height} != context.style().writing_mode.is_vertical() {
                                    return get_initial_value()
                                }
                            }
                        % endif
                        computed
                    }

                    #[inline]
                    fn from_computed_value(computed: &computed_value::T) -> Self {
                        SpecifiedValue(ToComputedValue::from_computed_value(computed))
                    }
                }
            </%helpers:longhand>
        % endfor
    % else:
        // servo versions (no keyword support)
        ${helpers.predefined_type("min-%s" % size,
                                  "LengthOrPercentage",
                                  "computed::LengthOrPercentage::Length(Au(0))",
                                  "parse_non_negative",
                                  needs_context=False,
                                  spec=spec % ("min-%s" % size),
                                  animatable=True, logical = logical)}
        ${helpers.predefined_type("max-%s" % size,
                                  "LengthOrPercentageOrNone",
                                  "computed::LengthOrPercentageOrNone::None",
                                  "parse_non_negative",
                                  needs_context=False,
                                  spec=spec % ("min-%s" % size),
                                  animatable=True, logical = logical)}
    % endif
% endfor

${helpers.single_keyword("box-sizing",
                         "content-box border-box",
                         extra_prefixes="moz webkit",
                         spec="https://drafts.csswg.org/css-ui/#propdef-box-sizing",
                         animatable=False)}

${helpers.single_keyword("object-fit", "fill contain cover none scale-down",
                         products="gecko", animatable=False,
                         spec="https://drafts.csswg.org/css-images/#propdef-object-fit")}

${helpers.predefined_type("object-position",
                          "Position",
                          "computed::Position::zero()",
                          products="gecko",
                          boxed="True",
                          spec="https://drafts.csswg.org/css-images-3/#the-object-position",
                          animatable=True)}

% for kind in ["row", "column"]:
    ${helpers.predefined_type("grid-%s-gap" % kind,
                              "LengthOrPercentage",
                              "computed::LengthOrPercentage::Length(Au(0))",
                              spec="https://drafts.csswg.org/css-grid/#propdef-grid-%s-gap" % kind,
                              animatable=True,
                              products="gecko")}

    % for range in ["start", "end"]:
        ${helpers.predefined_type("grid-%s-%s" % (kind, range),
                                  "GridLine",
                                  "Default::default()",
                                  animatable=False,
                                  spec="https://drafts.csswg.org/css-grid/#propdef-grid-%s-%s" % (kind, range),
                                  products="gecko",
                                  boxed=True)}
    % endfor

    // NOTE: According to the spec, this should handle multiple values of `<track-size>`,
    // but gecko supports only a single value
    ${helpers.predefined_type("grid-auto-%ss" % kind,
                              "TrackSize",
                              "Default::default()",
                              animatable=False,
                              spec="https://drafts.csswg.org/css-grid/#propdef-grid-auto-%ss" % kind,
                              products="gecko",
                              boxed=True)}
% endfor

<%helpers:longhand name="grid-auto-flow"
        spec="https://drafts.csswg.org/css-grid/#propdef-grid-auto-flow"
        products="gecko"
        animatable="False">
    use std::fmt;
    use style_traits::ToCss;
    use values::HasViewportPercentage;
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
