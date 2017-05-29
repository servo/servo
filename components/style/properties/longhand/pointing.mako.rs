/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

<%namespace name="helpers" file="/helpers.mako.rs" />

<% data.new_style_struct("Pointing", inherited=True, gecko_name="UserInterface") %>

<%helpers:longhand name="cursor" boxed="${product == 'gecko'}" animation_value_type="none"
  spec="https://drafts.csswg.org/css-ui/#cursor">
    pub use self::computed_value::T as SpecifiedValue;
    use values::computed::ComputedValueAsSpecified;
    #[cfg(feature = "gecko")]
    use values::specified::url::SpecifiedUrl;

    impl ComputedValueAsSpecified for SpecifiedValue {}
    no_viewport_percentage!(SpecifiedValue);

    pub mod computed_value {
        use std::fmt;
        use style_traits::cursor::Cursor;
        use style_traits::ToCss;
        #[cfg(feature = "gecko")]
        use values::specified::url::SpecifiedUrl;

        #[derive(Clone, PartialEq, Copy, Debug)]
        #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
        pub enum Keyword {
            AutoCursor,
            SpecifiedCursor(Cursor),
        }

        #[cfg(not(feature = "gecko"))]
        pub type T = Keyword;

        #[cfg(feature = "gecko")]
        #[derive(Clone, PartialEq, Debug)]
        pub struct Image {
            pub url: SpecifiedUrl,
            pub hotspot: Option<(f32, f32)>,
        }

        #[cfg(feature = "gecko")]
        #[derive(Clone, PartialEq, Debug)]
        pub struct T {
            pub images: Vec<Image>,
            pub keyword: Keyword,
        }

        impl ToCss for Keyword {
            fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
                match *self {
                    Keyword::AutoCursor => dest.write_str("auto"),
                    Keyword::SpecifiedCursor(c) => c.to_css(dest),
                }
            }
        }

        #[cfg(feature = "gecko")]
        impl ToCss for Image {
            fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
                try!(self.url.to_css(dest));
                if let Some((x, y)) = self.hotspot {
                    try!(dest.write_str(" "));
                    try!(x.to_css(dest));
                    try!(dest.write_str(" "));
                    try!(y.to_css(dest));
                }
                Ok(())
            }
        }

        #[cfg(feature = "gecko")]
        impl ToCss for T {
            fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
                for url in &self.images {
                    try!(url.to_css(dest));
                    try!(dest.write_str(", "));
                }
                self.keyword.to_css(dest)
            }
        }
    }

    #[cfg(not(feature = "gecko"))]
    #[inline]
    pub fn get_initial_value() -> computed_value::T {
        computed_value::Keyword::AutoCursor
    }

    #[cfg(feature = "gecko")]
    #[inline]
    pub fn get_initial_value() -> computed_value::T {
        computed_value::T {
            images: vec![],
            keyword: computed_value::Keyword::AutoCursor
        }
    }

    impl Parse for computed_value::Keyword {
        fn parse(_context: &ParserContext, input: &mut Parser) -> Result<computed_value::Keyword, ()> {
            use std::ascii::AsciiExt;
            use style_traits::cursor::Cursor;
            let ident = try!(input.expect_ident());
            if ident.eq_ignore_ascii_case("auto") {
                Ok(computed_value::Keyword::AutoCursor)
            } else {
                Cursor::from_css_keyword(&ident).map(computed_value::Keyword::SpecifiedCursor)
            }
        }
    }

    #[cfg(feature = "gecko")]
    fn parse_image(context: &ParserContext, input: &mut Parser) -> Result<computed_value::Image, ()> {
        Ok(computed_value::Image {
            url: try!(SpecifiedUrl::parse(context, input)),
            hotspot: match input.try(|input| input.expect_number()) {
                Ok(number) => Some((number, try!(input.expect_number()))),
                Err(()) => None,
            },
        })
    }

    #[cfg(not(feature = "gecko"))]
    pub fn parse(context: &ParserContext, input: &mut Parser) -> Result<SpecifiedValue, ()> {
        computed_value::Keyword::parse(context, input)
    }

    /// cursor: [<url> [<number> <number>]?]# [auto | default | ...]
    #[cfg(feature = "gecko")]
    pub fn parse(context: &ParserContext, input: &mut Parser) -> Result<SpecifiedValue, ()> {
        let mut images = vec![];
        loop {
            match input.try(|input| parse_image(context, input)) {
                Ok(mut image) => {
                    image.url.build_image_value();
                    images.push(image)
                }
                Err(()) => break,
            }
            try!(input.expect_comma());
        }

        Ok(computed_value::T {
            images: images,
            keyword: try!(computed_value::Keyword::parse(context, input)),
        })
    }
</%helpers:longhand>

// NB: `pointer-events: auto` (and use of `pointer-events` in anything that isn't SVG, in fact)
// is nonstandard, slated for CSS4-UI.
// TODO(pcwalton): SVG-only values.
${helpers.single_keyword("pointer-events", "auto none", animation_value_type="discrete",
                         extra_gecko_values="visiblepainted visiblefill visiblestroke visible painted fill stroke all",
                         spec="https://www.w3.org/TR/SVG11/interact.html#PointerEventsProperty")}

${helpers.single_keyword("-moz-user-input", "auto none enabled disabled",
                         products="gecko", gecko_ffi_name="mUserInput",
                         gecko_enum_prefix="StyleUserInput",
                         gecko_inexhaustive=True,
                         animation_value_type="discrete",
                         spec="Nonstandard (https://developer.mozilla.org/en-US/docs/Web/CSS/-moz-user-input)")}

${helpers.single_keyword("-moz-user-modify", "read-only read-write write-only",
                         products="gecko", gecko_ffi_name="mUserModify",
                         gecko_enum_prefix="StyleUserModify",
                         needs_conversion=True,
                         gecko_inexhaustive=True,
                         animation_value_type="discrete",
                         spec="Nonstandard (https://developer.mozilla.org/en-US/docs/Web/CSS/-moz-user-modify)")}

${helpers.single_keyword("-moz-user-focus",
                         "none ignore normal select-after select-before select-menu select-same select-all",
                         products="gecko", gecko_ffi_name="mUserFocus",
                         gecko_enum_prefix="StyleUserFocus",
                         gecko_inexhaustive=True,
                         animation_value_type="discrete",
                         spec="Nonstandard (https://developer.mozilla.org/en-US/docs/Web/CSS/-moz-user-focus)")}

${helpers.predefined_type("caret-color",
                          "ColorOrAuto",
                          "Either::Second(Auto)",
                          spec="https://drafts.csswg.org/css-ui/#caret-color",
                          animation_value_type="Either<IntermediateColor, Auto>",
                          boxed=True,
                          products="gecko")}
