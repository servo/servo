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
                         animatable=True,
                         spec="https://drafts.csswg.org/css-box/#propdef-visibility")}

// CSS Writing Modes Level 3
// https://drafts.csswg.org/css-writing-modes-3
${helpers.single_keyword("writing-mode",
                         "horizontal-tb vertical-rl vertical-lr",
                         experimental=True,
                         need_clone=True,
                         animatable=False,
                         spec="https://drafts.csswg.org/css-writing-modes/#propdef-writing-mode")}

${helpers.single_keyword("direction", "ltr rtl", need_clone=True, animatable=False,
                         spec="https://drafts.csswg.org/css-writing-modes/#propdef-direction")}

// FIXME(SimonSapin): Add 'mixed' and 'upright' (needs vertical text support)
// FIXME(SimonSapin): initial (first) value should be 'mixed', when that's implemented
// FIXME(bholley): sideways-right is needed as an alias to sideways in gecko.
${helpers.single_keyword("text-orientation",
                         "sideways",
                         experimental=True,
                         need_clone=True,
                         extra_gecko_values="mixed upright",
                         extra_servo_values="sideways-right sideways-left",
                         animatable=False,
                         spec="https://drafts.csswg.org/css-writing-modes/#propdef-text-orientation")}

// CSS Color Module Level 4
// https://drafts.csswg.org/css-color/
${helpers.single_keyword("color-adjust",
                         "economy exact", products="gecko",
                         animatable=False,
                         spec="https://drafts.csswg.org/css-color/#propdef-color-adjust")}

<% image_rendering_custom_consts = { "crisp-edges": "CRISPEDGES" } %>
// According to to CSS-IMAGES-3, `optimizespeed` and `optimizequality` are synonyms for `auto`
// And, firefox doesn't support `pixelated` yet (https://bugzilla.mozilla.org/show_bug.cgi?id=856337)
${helpers.single_keyword("image-rendering",
                         "auto crisp-edges",
                         extra_gecko_values="optimizespeed optimizequality",
                         extra_servo_values="pixelated",
                         custom_consts=image_rendering_custom_consts,
                         animatable=False,
                         spec="https://drafts.csswg.org/css-images/#propdef-image-rendering")}

// Image Orientation
<%helpers:longhand name="image-orientation"
                   products="None"
                   animatable="False"
    spec="https://drafts.csswg.org/css-images/#propdef-image-orientation, \
      /// additional values in https://developer.mozilla.org/en-US/docs/Web/CSS/image-orientation">
    use std::fmt;
    use style_traits::ToCss;
    use values::specified::Angle;

    use values::NoViewportPercentage;
    impl NoViewportPercentage for SpecifiedValue {}

    use std::f32::consts::PI;
    use values::CSSFloat;
    const TWO_PI: CSSFloat = 2.0*PI;

    #[derive(Clone, PartialEq, Copy, Debug)]
    #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
    pub struct SpecifiedValue {
        pub angle: Option<Angle>,
        pub flipped: bool
    }

    impl ToCss for SpecifiedValue {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            if let Some(angle) = self.angle {
                try!(angle.to_css(dest));
                if self.flipped {
                    dest.write_str(" flipped")
                } else {
                    Ok(())
                }
            } else {
                if self.flipped {
                    dest.write_str("flipped")
                } else {
                    dest.write_str("from-image")
                }
            }
        }
    }

    pub mod computed_value {
        use values::specified::Angle;

        #[derive(Clone, PartialEq, Copy, Debug)]
        #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
        pub enum T {
            FromImage,
            AngleWithFlipped(Angle, bool),
        }
    }

    const INITIAL_ANGLE: Angle = Angle(0.0);

    #[inline]
    pub fn get_initial_value() -> computed_value::T {
        computed_value::T::AngleWithFlipped(INITIAL_ANGLE, false)
    }

    // According to CSS Content Module Level 3:
    // The computed value of the property is calculated by rounding the specified angle
    // to the nearest quarter-turn, rounding away from 0, then moduloing the value by 1 turn.
    #[inline]
    fn normalize_angle(angle: &Angle) -> Angle {
        let radians = angle.radians();
        let rounded_quarter_turns = (4.0 * radians / TWO_PI).round();
        let normalized_quarter_turns = (rounded_quarter_turns % 4.0 + 4.0) % 4.0;
        let normalized_radians = normalized_quarter_turns/4.0 * TWO_PI;
        Angle::from_radians(normalized_radians)
    }

    impl ToComputedValue for SpecifiedValue {
        type ComputedValue = computed_value::T;

        #[inline]
        fn to_computed_value(&self, _: &Context) -> computed_value::T {
            if let Some(ref angle) = self.angle {
                let normalized_angle = normalize_angle(angle);
                computed_value::T::AngleWithFlipped(normalized_angle, self.flipped)
            } else {
                if self.flipped {
                    computed_value::T::AngleWithFlipped(INITIAL_ANGLE, true)
                } else {
                    computed_value::T::FromImage
                }
            }
        }

        #[inline]
        fn from_computed_value(computed: &computed_value::T) -> Self {
            match *computed {
                computed_value::T::FromImage => SpecifiedValue { angle: None, flipped: false },
                computed_value::T::AngleWithFlipped(angle, flipped) =>
                    SpecifiedValue { angle: Some(angle), flipped: flipped },
            }
        }
    }

    impl ToCss for computed_value::T {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            match *self {
                computed_value::T::FromImage => dest.write_str("from-image"),
                computed_value::T::AngleWithFlipped(angle, flipped) => {
                    try!(angle.to_css(dest));
                    if flipped {
                        try!(dest.write_str(" flipped"));
                    }
                    Ok(())
                },
            }
        }
    }

    pub fn parse(context: &ParserContext, input: &mut Parser) -> Result<SpecifiedValue, ()> {
        if input.try(|input| input.expect_ident_matching("from-image")).is_ok() {
            // Handle from-image
            Ok(SpecifiedValue { angle: None, flipped: false })
        } else {
            // Handle <angle> | <angle>? flip
            let angle = input.try(|input| Angle::parse(context, input)).ok();
            let flipped = input.try(|input| input.expect_ident_matching("flip")).is_ok();
            let explicit_angle = if angle.is_none() && !flipped {
                Some(INITIAL_ANGLE)
            } else {
                angle
            };

            Ok(SpecifiedValue { angle: explicit_angle, flipped: flipped })
        }
    }
</%helpers:longhand>

// Used in the bottom-up flow construction traversal to avoid constructing flows for
// descendants of nodes with `display: none`.
<%helpers:longhand name="-servo-under-display-none"
                   derived_from="display"
                   products="servo"
                   animatable="False"
                   spec="Nonstandard (internal layout use only)">
    use std::fmt;
    use style_traits::ToCss;
    use values::computed::ComputedValueAsSpecified;
    use values::NoViewportPercentage;

    impl NoViewportPercentage for SpecifiedValue {}

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
