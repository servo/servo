/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

<%namespace name="helpers" file="/helpers.mako.rs" />

// SVG 1.1 (Second Edition)
// https://www.w3.org/TR/SVG/
<% data.new_style_struct("InheritedSVG",
                         inherited=True,
                         gecko_name="SVG") %>

// Section 10 - Text

${helpers.single_keyword("text-anchor",
                         "start middle end",
                         products="gecko",
                         animation_value_type="discrete",
                         spec="https://www.w3.org/TR/SVG/text.html#TextAnchorProperty")}

// Section 11 - Painting: Filling, Stroking and Marker Symbols
${helpers.single_keyword("color-interpolation",
                         "srgb auto linearrgb",
                         products="gecko",
                         animation_value_type="discrete",
                         spec="https://www.w3.org/TR/SVG11/painting.html#ColorInterpolationProperty")}

${helpers.single_keyword("color-interpolation-filters", "linearrgb auto srgb",
                         products="gecko",
                         gecko_constant_prefix="NS_STYLE_COLOR_INTERPOLATION",
                         animation_value_type="discrete",
                         spec="https://www.w3.org/TR/SVG11/painting.html#ColorInterpolationFiltersProperty")}

${helpers.predefined_type(
    "fill", "SVGPaint",
    "::values::computed::SVGPaint::black()",
    products="gecko",
    animation_value_type="IntermediateSVGPaint",
    boxed=True,
    spec="https://www.w3.org/TR/SVG2/painting.html#SpecifyingFillPaint")}

${helpers.predefined_type("fill-opacity", "SVGOpacity", "Default::default()",
                          products="gecko", animation_value_type="ComputedValue",
                          spec="https://www.w3.org/TR/SVG11/painting.html#FillOpacityProperty")}

${helpers.single_keyword("fill-rule", "nonzero evenodd",
                         gecko_enum_prefix="StyleFillRule",
                         products="gecko", animation_value_type="discrete",
                         spec="https://www.w3.org/TR/SVG11/painting.html#FillRuleProperty")}

${helpers.single_keyword("shape-rendering",
                         "auto optimizespeed crispedges geometricprecision",
                         products="gecko",
                         animation_value_type="discrete",
                         spec="https://www.w3.org/TR/SVG11/painting.html#ShapeRenderingProperty")}

${helpers.predefined_type(
    "stroke", "SVGPaint",
    "Default::default()",
    products="gecko",
    animation_value_type="IntermediateSVGPaint",
    boxed=True,
    spec="https://www.w3.org/TR/SVG2/painting.html#SpecifyingStrokePaint")}

${helpers.predefined_type(
    "stroke-width", "SVGWidth",
    "::values::computed::NonNegativeLength::new(1.).into()",
    products="gecko",
    boxed="True",
    animation_value_type="::values::computed::SVGWidth",
    spec="https://www.w3.org/TR/SVG2/painting.html#StrokeWidth")}

${helpers.single_keyword("stroke-linecap", "butt round square",
                         products="gecko", animation_value_type="discrete",
                         spec="https://www.w3.org/TR/SVG11/painting.html#StrokeLinecapProperty")}

${helpers.single_keyword("stroke-linejoin", "miter round bevel",
                         products="gecko", animation_value_type="discrete",
                         spec="https://www.w3.org/TR/SVG11/painting.html#StrokeLinejoinProperty")}

${helpers.predefined_type("stroke-miterlimit", "GreaterThanOrEqualToOneNumber",
                          "From::from(4.0)",
                          products="gecko",
                          animation_value_type="::values::computed::GreaterThanOrEqualToOneNumber",
                          spec="https://www.w3.org/TR/SVG11/painting.html#StrokeMiterlimitProperty")}

${helpers.predefined_type("stroke-opacity", "SVGOpacity", "Default::default()",
                          products="gecko", animation_value_type="ComputedValue",
                          spec="https://www.w3.org/TR/SVG11/painting.html#StrokeOpacityProperty")}

${helpers.predefined_type(
    "stroke-dasharray",
    "SVGStrokeDashArray",
    "Default::default()",
    products="gecko",
    animation_value_type="::values::computed::SVGStrokeDashArray",
    spec="https://www.w3.org/TR/SVG2/painting.html#StrokeDashing",
)}

${helpers.predefined_type(
    "stroke-dashoffset", "SVGLength",
    "Au(0).into()",
    products="gecko",
    boxed="True",
    animation_value_type="ComputedValue",
    spec="https://www.w3.org/TR/SVG2/painting.html#StrokeDashing")}

// Section 14 - Clipping, Masking and Compositing
${helpers.single_keyword("clip-rule", "nonzero evenodd",
                         products="gecko",
                         gecko_enum_prefix="StyleFillRule",
                         animation_value_type="discrete",
                         spec="https://www.w3.org/TR/SVG11/masking.html#ClipRuleProperty")}

${helpers.predefined_type("marker-start", "UrlOrNone", "Either::Second(None_)",
                          products="gecko",
                          boxed="True" if product == "gecko" else "False",
                          animation_value_type="discrete",
                          spec="https://www.w3.org/TR/SVG2/painting.html#VertexMarkerProperties")}

${helpers.predefined_type("marker-mid", "UrlOrNone", "Either::Second(None_)",
                          products="gecko",
                          boxed="True" if product == "gecko" else "False",
                          animation_value_type="discrete",
                          spec="https://www.w3.org/TR/SVG2/painting.html#VertexMarkerProperties")}

${helpers.predefined_type("marker-end", "UrlOrNone", "Either::Second(None_)",
                          products="gecko",
                          boxed="True" if product == "gecko" else "False",
                          animation_value_type="discrete",
                          spec="https://www.w3.org/TR/SVG2/painting.html#VertexMarkerProperties")}

<%helpers:longhand name="paint-order"
                   animation_value_type="discrete"
                   products="gecko"
                   spec="https://www.w3.org/TR/SVG2/painting.html#PaintOrder">
    use std::fmt;
    use style_traits::ToCss;

    /// The specified value for a single CSS paint-order property.
    #[repr(u8)]
    #[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, ToCss)]
    pub enum PaintOrder {
        Normal = 0,
        Fill = 1,
        Stroke = 2,
        Markers = 3,
    }

    /// Number of non-normal components.
    const COUNT: u8 = 3;

    /// Number of bits for each component
    const SHIFT: u8 = 2;

    /// Mask with above bits set
    const MASK: u8 = 0b11;

    /// The specified value is tree `PaintOrder` values packed into the
    /// bitfields below, as a six-bit field, of 3 two-bit pairs
    ///
    /// Each pair can be set to FILL, STROKE, or MARKERS
    /// Lowest significant bit pairs are highest priority.
    ///  `normal` is the empty bitfield. The three pairs are
    /// never zero in any case other than `normal`.
    ///
    /// Higher priority values, i.e. the values specified first,
    /// will be painted first (and may be covered by paintings of lower priority)
    #[derive(Clone, Copy, Debug, MallocSizeOf, PartialEq, ToComputedValue)]
    pub struct SpecifiedValue(pub u8);

    impl SpecifiedValue {
        fn normal() -> Self {
            SpecifiedValue(0)
        }
    }

    pub mod computed_value {
        pub use super::SpecifiedValue as T;
    }

    pub fn get_initial_value() -> SpecifiedValue {
        SpecifiedValue::normal()
    }

    impl SpecifiedValue {
        fn order_at(&self, pos: u8) -> PaintOrder {
            // Safe because PaintOrder covers all possible patterns.
            unsafe { ::std::mem::transmute((self.0 >> pos * SHIFT) & MASK) }
        }
    }

    pub fn parse<'i, 't>(
        _context: &ParserContext,
        input: &mut Parser<'i, 't>
    ) -> Result<SpecifiedValue,ParseError<'i>> {
        if let Ok(()) = input.try(|i| i.expect_ident_matching("normal")) {
            return Ok(SpecifiedValue::normal())
        }

        let mut value = 0;
        // bitfield representing what we've seen so far
        // bit 1 is fill, bit 2 is stroke, bit 3 is markers
        let mut seen = 0;
        let mut pos = 0;

        loop {
            let result: Result<_, ParseError> = input.try(|input| {
                try_match_ident_ignore_ascii_case! { input,
                    "fill" => Ok(PaintOrder::Fill),
                    "stroke" => Ok(PaintOrder::Stroke),
                    "markers" => Ok(PaintOrder::Markers),
                }
            });

            match result {
                Ok(val) => {
                    if (seen & (1 << val as u8)) != 0 {
                        // don't parse the same ident twice
                        return Err(input.new_custom_error(StyleParseErrorKind::UnspecifiedError))
                    }

                    value |= (val as u8) << (pos * SHIFT);
                    seen |= 1 << (val as u8);
                    pos += 1;
                }
                Err(_) => break,
            }
        }

        if value == 0 {
            // Couldn't find any keyword
            return Err(input.new_custom_error(StyleParseErrorKind::UnspecifiedError))
        }

        // fill in rest
        for i in pos..COUNT {
            for paint in 0..COUNT {
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

    impl ToCss for SpecifiedValue {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            if self.0 == 0 {
                return dest.write_str("normal")
            }

            let mut last_pos_to_serialize = 0;
            for i in (1..COUNT).rev() {
                let component = self.order_at(i);
                let earlier_component = self.order_at(i - 1);
                if component < earlier_component {
                    last_pos_to_serialize = i - 1;
                    break;
                }
            }

            for pos in 0..last_pos_to_serialize + 1 {
                if pos != 0 {
                    dest.write_str(" ")?
                }
                self.order_at(pos).to_css(dest)?;
            }
            Ok(())
        }
    }
</%helpers:longhand>

<%helpers:vector_longhand name="-moz-context-properties"
                   animation_value_type="none"
                   products="gecko"
                   spec="Nonstandard (Internal-only)"
                   allow_empty="True">
    use values::CustomIdent;

    pub type SpecifiedValue = CustomIdent;

    pub mod computed_value {
        pub type T = super::SpecifiedValue;
    }


    pub fn parse<'i, 't>(_context: &ParserContext, input: &mut Parser<'i, 't>)
                         -> Result<SpecifiedValue, ParseError<'i>> {
        let location = input.current_source_location();
        let i = input.expect_ident()?;
        CustomIdent::from_ident(location, i, &["all", "none", "auto"])
    }
</%helpers:vector_longhand>
