/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

<%namespace name="helpers" file="/helpers.mako.rs" />

// SVG 1.1 (Second Edition)
// https://www.w3.org/TR/SVG/
<% data.new_style_struct("InheritedSVG",
                         inherited=True,
                         gecko_name="SVG") %>

// TODO(emilio): Should some of these types be animatable?

// Section 10 - Text

${helpers.single_keyword("text-anchor",
                         "start middle end",
                         products="gecko",
                         animation_type="none",
                         spec="https://www.w3.org/TR/SVG/text.html#TextAnchorProperty")}

// Section 11 - Painting: Filling, Stroking and Marker Symbols
${helpers.single_keyword("color-interpolation",
                         "srgb auto linearrgb",
                         products="gecko",
                         animation_type="none",
                         spec="https://www.w3.org/TR/SVG11/painting.html#ColorInterpolationProperty")}

${helpers.single_keyword("color-interpolation-filters", "linearrgb auto srgb",
                         products="gecko",
                         gecko_constant_prefix="NS_STYLE_COLOR_INTERPOLATION",
                         animation_type="none",
                         spec="https://www.w3.org/TR/SVG11/painting.html#ColorInterpolationFiltersProperty")}

${helpers.predefined_type(
    "fill", "SVGPaint",
    "::values::computed::SVGPaint::black()",
    products="gecko",
    animation_type="none",
    boxed=True,
    spec="https://www.w3.org/TR/SVG2/painting.html#SpecifyingFillPaint")}

${helpers.predefined_type("fill-opacity", "Opacity", "1.0",
                          products="gecko", animation_type="none",
                          spec="https://www.w3.org/TR/SVG11/painting.html#FillOpacityProperty")}

${helpers.single_keyword("fill-rule", "nonzero evenodd",
                         gecko_enum_prefix="StyleFillRule",
                         gecko_inexhaustive=True,
                         products="gecko", animation_type="none",
                         spec="https://www.w3.org/TR/SVG11/painting.html#FillRuleProperty")}

${helpers.single_keyword("shape-rendering",
                         "auto optimizespeed crispedges geometricprecision",
                         products="gecko",
                         animation_type="none",
                         spec="https://www.w3.org/TR/SVG11/painting.html#ShapeRenderingProperty")}

${helpers.predefined_type(
    "stroke", "SVGPaint",
    "Default::default()",
    products="gecko",
    animation_type="none",
    boxed=True,
    spec="https://www.w3.org/TR/SVG2/painting.html#SpecifyingStrokePaint")}

${helpers.predefined_type(
    "stroke-width", "LengthOrPercentage",
    "computed::LengthOrPercentage::one()",
    "parse_numbers_are_pixels_non_negative",
    products="gecko",
    animation_type="normal",
    spec="https://www.w3.org/TR/SVG2/painting.html#StrokeWidth")}

${helpers.single_keyword("stroke-linecap", "butt round square",
                         products="gecko", animation_type="none",
                         spec="https://www.w3.org/TR/SVG11/painting.html#StrokeLinecapProperty")}

${helpers.single_keyword("stroke-linejoin", "miter round bevel",
                         products="gecko", animation_type="none",
                         spec="https://www.w3.org/TR/SVG11/painting.html#StrokeLinejoinProperty")}

${helpers.predefined_type("stroke-miterlimit", "Number", "4.0",
                          "parse_at_least_one", products="gecko",
                          animation_type="none",
                          spec="https://www.w3.org/TR/SVG11/painting.html#StrokeMiterlimitProperty")}

${helpers.predefined_type("stroke-opacity", "Opacity", "1.0",
                          products="gecko", animation_type="none",
                          spec="https://www.w3.org/TR/SVG11/painting.html#StrokeOpacityProperty")}

${helpers.predefined_type("stroke-dasharray",
                          "LengthOrPercentageOrNumber",
                          "Either::Second(0.0)",
                          "parse_non_negative",
                          vector="True",
                          allow_empty="True",
                          products="gecko",
                          animation_type="none",
                          space_separated_allowed="True",
                          spec="https://www.w3.org/TR/SVG2/painting.html#StrokeDashing")}

${helpers.predefined_type(
    "stroke-dashoffset", "LengthOrPercentage",
    "computed::LengthOrPercentage::zero()",
    "parse_numbers_are_pixels",
    products="gecko",
    animation_type="normal",
    spec="https://www.w3.org/TR/SVG2/painting.html#StrokeDashing")}

// Section 14 - Clipping, Masking and Compositing
${helpers.single_keyword("clip-rule", "nonzero evenodd",
                         products="gecko",
                         gecko_enum_prefix="StyleFillRule",
                         gecko_inexhaustive=True,
                         animation_type="none",
                         spec="https://www.w3.org/TR/SVG11/masking.html#ClipRuleProperty")}

${helpers.predefined_type("marker-start", "UrlOrNone", "Either::Second(None_)",
                          products="gecko",
                          animation_type="none",
                          spec="https://www.w3.org/TR/SVG2/painting.html#VertexMarkerProperties")}

${helpers.predefined_type("marker-mid", "UrlOrNone", "Either::Second(None_)",
                          products="gecko",
                          animation_type="none",
                          spec="https://www.w3.org/TR/SVG2/painting.html#VertexMarkerProperties")}

${helpers.predefined_type("marker-end", "UrlOrNone", "Either::Second(None_)",
                          products="gecko",
                          animation_type="none",
                          spec="https://www.w3.org/TR/SVG2/painting.html#VertexMarkerProperties")}

<%helpers:longhand name="paint-order"
                   animation_type="none"
                   products="gecko"
                   spec="https://www.w3.org/TR/SVG2/painting.html#PaintOrder">

    use values::computed::ComputedValueAsSpecified;
    use std::fmt;
    use style_traits::ToCss;
    use values::HasViewportPercentage;

    pub const NORMAL: u8 = 0;
    pub const FILL: u8 = 1;
    pub const STROKE: u8 = 2;
    pub const MARKERS: u8 = 3;

    // number of bits for each component
    pub const SHIFT: u8 = 2;
    // mask with above bits set
    pub const MASK: u8 = 0b11;
    // number of non-normal keyword values
    pub const COUNT: u8 = 3;
    // all keywords
    pub const ALL: [u8; 3] = [FILL, STROKE, MARKERS];

    /// Represented as a six-bit field, of 3 two-bit pairs
    ///
    /// Each pair can be set to FILL, STROKE, or MARKERS
    /// Lowest significant bit pairs are highest priority.
    ///  `normal` is the empty bitfield. The three pairs are
    /// never zero in any case other than `normal`.
    ///
    /// Higher priority values, i.e. the values specified first,
    /// will be painted first (and may be covered by paintings of lower priority)
    #[derive(PartialEq, Clone, Copy, Debug)]
    #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
    pub struct SpecifiedValue(pub u8);

    pub mod computed_value {
        pub use super::SpecifiedValue as T;
    }

    pub fn get_initial_value() -> SpecifiedValue {
      SpecifiedValue(NORMAL)
    }

    impl SpecifiedValue {
        pub fn bits_at(&self, pos: u8) -> u8 {
            (self.0 >> pos * SHIFT) & MASK
        }
    }

    pub fn parse(_context: &ParserContext, input: &mut Parser) -> Result<SpecifiedValue,()> {
        if let Ok(()) = input.try(|i| i.expect_ident_matching("normal")) {
            Ok(SpecifiedValue(0))
        } else {
            let mut value = 0;
            // bitfield representing what we've seen so far
            // bit 1 is fill, bit 2 is stroke, bit 3 is markers
            let mut seen = 0;
            let mut pos = 0;

            loop {

                let result = input.try(|i| {
                    match_ignore_ascii_case! { &i.expect_ident()?,
                        "fill" => Ok(FILL),
                        "stroke" => Ok(STROKE),
                        "markers" => Ok(MARKERS),
                        _ => Err(())
                    }
                });

                match result {
                    Ok(val) => {
                        if (seen & (1 << val)) != 0 {
                            // don't parse the same ident twice
                            return Err(())
                        } else {
                            value |= val << (pos * SHIFT);
                            seen |= 1 << val;
                            pos += 1;
                        }
                    }
                    Err(()) => break,
                }
            }

            if value == 0 {
                // couldn't find any keyword
                Err(())
            } else {
                // fill in rest
                for i in pos..COUNT {
                    for paint in &ALL {
                        // if not seen, set bit at position, mark as seen
                        if (seen & (1 << paint)) == 0 {
                            seen |= 1 << paint;
                            value |= paint << (i * SHIFT);
                            break;
                        }
                    }
                }

                Ok(SpecifiedValue(value))
            }
        }
    }

    impl ToCss for SpecifiedValue {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            if self.0 == 0 {
                return dest.write_str("normal")
            }

            for pos in 0..COUNT {
                if pos != 0 {
                    dest.write_str(" ")?
                }
                match self.bits_at(pos) {
                    FILL => dest.write_str("fill")?,
                    STROKE => dest.write_str("stroke")?,
                    MARKERS => dest.write_str("markers")?,
                    _ => unreachable!(),
                }
            }
            Ok(())
        }
    }

    no_viewport_percentage!(SpecifiedValue);

    impl ComputedValueAsSpecified for SpecifiedValue { }
</%helpers:longhand>

