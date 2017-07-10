/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

<%namespace name="helpers" file="/helpers.mako.rs" />

<% data.new_style_struct("InheritedBox", inherited=True, gecko_name="Visibility") %>

// TODO: collapse. Well, do tables first.
${helpers.single_keyword("visibility",
                         "visible hidden",
                         extra_gecko_values="collapse",
                         gecko_ffi_name="mVisible",
                         animation_value_type="ComputedValue",
                         spec="https://drafts.csswg.org/css-box/#propdef-visibility")}

// CSS Writing Modes Level 3
// https://drafts.csswg.org/css-writing-modes-3
${helpers.single_keyword("writing-mode",
                         "horizontal-tb vertical-rl vertical-lr",
                         extra_gecko_values="sideways-rl sideways-lr",
                         extra_gecko_aliases="lr=horizontal-tb lr-tb=horizontal-tb \
                                              rl=horizontal-tb rl-tb=horizontal-tb \
                                              tb=vertical-rl   tb-rl=vertical-rl",
                         experimental=True,
                         animation_value_type="discrete",
                         spec="https://drafts.csswg.org/css-writing-modes/#propdef-writing-mode")}

${helpers.single_keyword("direction", "ltr rtl", animation_value_type="discrete",
                         spec="https://drafts.csswg.org/css-writing-modes/#propdef-direction",
                         needs_conversion=True)}

${helpers.single_keyword("text-orientation",
                         "mixed upright sideways",
                         extra_gecko_aliases="sideways-right=sideways",
                         products="gecko",
                         animation_value_type="discrete",
                         spec="https://drafts.csswg.org/css-writing-modes/#propdef-text-orientation")}

// CSS Color Module Level 4
// https://drafts.csswg.org/css-color/
${helpers.single_keyword("color-adjust",
                         "economy exact", products="gecko",
                         animation_value_type="discrete",
                         spec="https://drafts.csswg.org/css-color/#propdef-color-adjust")}

<% image_rendering_custom_consts = { "crisp-edges": "CRISPEDGES",
                                     "-moz-crisp-edges": "CRISPEDGES" } %>
// According to to CSS-IMAGES-3, `optimizespeed` and `optimizequality` are synonyms for `auto`
// And, firefox doesn't support `pixelated` yet (https://bugzilla.mozilla.org/show_bug.cgi?id=856337)
${helpers.single_keyword("image-rendering",
                         "auto",
                         extra_gecko_values="optimizespeed optimizequality -moz-crisp-edges",
                         extra_servo_values="pixelated crisp-edges",
                         custom_consts=image_rendering_custom_consts,
                         animation_value_type="discrete",
                         spec="https://drafts.csswg.org/css-images/#propdef-image-rendering")}

// Image Orientation
<%helpers:longhand name="image-orientation"
                   products="gecko"
                   animation_value_type="none"
    spec="https://drafts.csswg.org/css-images/#propdef-image-orientation, \
      /// additional values in https://developer.mozilla.org/en-US/docs/Web/CSS/image-orientation">
    use std::fmt;
    use style_traits::ToCss;
    use values::specified::Angle;

    no_viewport_percentage!(SpecifiedValue);

    use std::f64::consts::PI;
    const TWO_PI: f64 = 2.0 * PI;

    #[derive(Clone, PartialEq, Copy, Debug)]
    #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
    pub struct SpecifiedValue {
        pub angle: Option<Angle>,
        pub flipped: bool
    }

    impl ToCss for SpecifiedValue {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            if let Some(angle) = self.angle {
                angle.to_css(dest)?;
                if self.flipped {
                    dest.write_str(" flip")
                } else {
                    Ok(())
                }
            } else {
                if self.flipped {
                    dest.write_str("flip")
                } else {
                    dest.write_str("from-image")
                }
            }
        }
    }

    pub mod computed_value {
        use std::fmt;
        use style_traits::ToCss;
        use values::specified::Angle;

        #[derive(Copy, Clone, Debug, Eq, PartialEq)]
        #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
        pub enum Orientation {
            Angle0 = 0,
            Angle90,
            Angle180,
            Angle270,
        }

        impl Orientation {
            pub fn angle(&self) -> Angle {
                match *self {
                    Orientation::Angle0 => Angle::from_degrees(0.0, false),
                    Orientation::Angle90 => Angle::from_degrees(90.0, false),
                    Orientation::Angle180 => Angle::from_degrees(180.0, false),
                    Orientation::Angle270 => Angle::from_degrees(270.0, false),
                }
            }
        }

        impl ToCss for Orientation {
            fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
                // Should agree with Angle::to_css.
                match *self {
                    Orientation::Angle0 => dest.write_str("0deg"),
                    Orientation::Angle90 => dest.write_str("90deg"),
                    Orientation::Angle180 => dest.write_str("180deg"),
                    Orientation::Angle270 => dest.write_str("270deg"),
                }
            }
        }

        #[derive(Clone, PartialEq, Copy, Debug)]
        #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
        pub enum T {
            FromImage,
            AngleWithFlipped(Orientation, bool),
        }
    }

    use self::computed_value::Orientation;

    #[inline]
    pub fn get_initial_value() -> computed_value::T {
        computed_value::T::AngleWithFlipped(Orientation::Angle0, false)
    }

    // According to CSS Content Module Level 3:
    // The computed value of the property is calculated by rounding the specified angle
    // to the nearest quarter-turn, rounding away from 0, then moduloing the value by 1 turn.
    // This mirrors the Gecko implementation in
    // nsStyleImageOrientation::CreateAsAngleAndFlip.
    #[inline]
    fn orientation_of_angle(angle: &computed::Angle) -> Orientation {
        // Note that `angle` can be negative.
        let mut rounded_angle = angle.radians64() % TWO_PI;
        if rounded_angle < 0.0 {
            // This computation introduces rounding error. Gecko previously
            // didn't handle the negative case correctly; by branching we can
            // match Gecko's behavior when it was correct.
            rounded_angle = rounded_angle + TWO_PI;
        }
        if      rounded_angle < 0.25 * PI { Orientation::Angle0   }
        else if rounded_angle < 0.75 * PI { Orientation::Angle90  }
        else if rounded_angle < 1.25 * PI { Orientation::Angle180 }
        else if rounded_angle < 1.75 * PI { Orientation::Angle270 }
        else                              { Orientation::Angle0   }
    }

    impl ToComputedValue for SpecifiedValue {
        type ComputedValue = computed_value::T;

        #[inline]
        fn to_computed_value(&self, context: &Context) -> computed_value::T {
            if let Some(ref angle) = self.angle {
                let angle = angle.to_computed_value(context);
                let orientation = orientation_of_angle(&angle);
                computed_value::T::AngleWithFlipped(orientation, self.flipped)
            } else {
                if self.flipped {
                    computed_value::T::AngleWithFlipped(Orientation::Angle0, true)
                } else {
                    computed_value::T::FromImage
                }
            }
        }

        #[inline]
        fn from_computed_value(computed: &computed_value::T) -> Self {
            match *computed {
                computed_value::T::FromImage => SpecifiedValue { angle: None, flipped: false },
                computed_value::T::AngleWithFlipped(ref orientation, flipped) => {
                    SpecifiedValue {
                        angle: Some(orientation.angle()),
                        flipped: flipped,
                    }
                }
            }
        }
    }

    impl ToCss for computed_value::T {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            match *self {
                computed_value::T::FromImage => dest.write_str("from-image"),
                computed_value::T::AngleWithFlipped(angle, flipped) => {
                    angle.to_css(dest)?;
                    if flipped {
                        dest.write_str(" flip")?;
                    }
                    Ok(())
                },
            }
        }
    }

    // from-image | <angle> | [<angle>? flip]
    pub fn parse<'i, 't>(context: &ParserContext, input: &mut Parser<'i, 't>)
                         -> Result<SpecifiedValue, ParseError<'i>> {
        if input.try(|input| input.expect_ident_matching("from-image")).is_ok() {
            // Handle from-image
            Ok(SpecifiedValue { angle: None, flipped: false })
        } else if input.try(|input| input.expect_ident_matching("flip")).is_ok() {
            // Handle flip
            Ok(SpecifiedValue { angle: Some(Angle::zero()), flipped: true })
        } else {
            // Handle <angle> | <angle> flip
            let angle = input.try(|input| Angle::parse(context, input)).ok();
            if angle.is_none() {
                return Err(StyleParseError::UnspecifiedError.into());
            }

            let flipped = input.try(|input| input.expect_ident_matching("flip")).is_ok();
            Ok(SpecifiedValue { angle: angle, flipped: flipped })
        }
    }
</%helpers:longhand>

// Used in the bottom-up flow construction traversal to avoid constructing flows for
// descendants of nodes with `display: none`.
<%helpers:longhand name="-servo-under-display-none"
                   derived_from="display"
                   products="servo"
                   animation_value_type="none"
                   spec="Nonstandard (internal layout use only)">
    use std::fmt;
    use style_traits::ToCss;
    use values::computed::ComputedValueAsSpecified;

    no_viewport_percentage!(SpecifiedValue);

    #[derive(Copy, Clone, Debug, Eq, PartialEq)]
    #[cfg_attr(feature = "servo", derive(HeapSizeOf, Deserialize, Serialize))]
    pub struct SpecifiedValue(pub bool);

    pub mod computed_value {
        pub type T = super::SpecifiedValue;
    }
    impl ComputedValueAsSpecified for SpecifiedValue {}

    pub fn get_initial_value() -> computed_value::T {
        SpecifiedValue(false)
    }

    impl ToCss for SpecifiedValue {
        fn to_css<W>(&self, _: &mut W) -> fmt::Result where W: fmt::Write {
            Ok(()) // Internal property
        }
    }

    #[inline]
    pub fn derive_from_display(context: &mut Context) {
        use super::display::computed_value::T as Display;

        if context.style().get_box().clone_display() == Display::none {
            context.mutate_style().mutate_inheritedbox()
                                  .set__servo_under_display_none(SpecifiedValue(true));
        }
    }
</%helpers:longhand>
