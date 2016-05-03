/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

<%namespace name="helpers" file="/helpers.mako.rs" />

<% data.new_style_struct("InheritedBox", inherited=True, gecko_name="Visibility") %>

${helpers.single_keyword("direction", "ltr rtl", need_clone=True)}

// TODO: collapse. Well, do tables first.
${helpers.single_keyword("visibility",
                         "visible hidden",
                         extra_gecko_values="collapse",
                         gecko_ffi_name="mVisible")}

// CSS Writing Modes Level 3
// http://dev.w3.org/csswg/css-writing-modes/
${helpers.single_keyword("writing-mode",
                         "horizontal-tb vertical-rl vertical-lr",
                         experimental=True,
                         need_clone=True)}

// FIXME(SimonSapin): Add 'mixed' and 'upright' (needs vertical text support)
// FIXME(SimonSapin): initial (first) value should be 'mixed', when that's implemented
// FIXME(bholley): sideways-right is needed as an alias to sideways in gecko.
${helpers.single_keyword("text-orientation",
                         "sideways",
                         experimental=True,
                         need_clone=True,
                         extra_gecko_values="mixed upright",
                         extra_servo_values="sideways-right sideways-left")}

// CSS Color Module Level 4
// https://drafts.csswg.org/css-color/
${helpers.single_keyword("color-adjust", "economy exact", products="gecko")}

<%helpers:longhand name="image-rendering">
    pub mod computed_value {
        use cssparser::ToCss;
        use std::fmt;

        #[derive(Copy, Clone, Debug, PartialEq, HeapSizeOf, Deserialize, Serialize)]
        pub enum T {
            Auto,
            CrispEdges,
            Pixelated,
        }

        impl ToCss for T {
            fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
                match *self {
                    T::Auto => dest.write_str("auto"),
                    T::CrispEdges => dest.write_str("crisp-edges"),
                    T::Pixelated => dest.write_str("pixelated"),
                }
            }
        }
    }

    pub type SpecifiedValue = computed_value::T;

    #[inline]
    pub fn get_initial_value() -> computed_value::T {
        computed_value::T::Auto
    }

    pub fn parse(_: &ParserContext, input: &mut Parser) -> Result<SpecifiedValue,()> {
        // According to to CSS-IMAGES-3, `optimizespeed` and `optimizequality` are synonyms for
        // `auto`.
        match_ignore_ascii_case! {
            try!(input.expect_ident()),
            "auto" => Ok(computed_value::T::Auto),
            "optimizespeed" => Ok(computed_value::T::Auto),
            "optimizequality" => Ok(computed_value::T::Auto),
            "crisp-edges" => Ok(computed_value::T::CrispEdges),
            "pixelated" => Ok(computed_value::T::Pixelated),
            _ => Err(())
        }
    }

    impl ToComputedValue for SpecifiedValue {
        type ComputedValue = computed_value::T;

        #[inline]
        fn to_computed_value<Cx: TContext>(&self, _: &Cx) -> computed_value::T {
            *self
        }
    }
</%helpers:longhand>
