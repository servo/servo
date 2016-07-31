/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

<%namespace name="helpers" file="/helpers.mako.rs" />

<%helpers:shorthand name="font" sub_properties="font-style font-variant font-weight
                                                font-size line-height font-family">
    use properties::longhands::{font_style, font_variant, font_weight, font_size,
                                line_height, font_family};

    pub fn parse_value(context: &ParserContext, input: &mut Parser) -> Result<Longhands, ()> {
        let mut nb_normals = 0;
        let mut style = None;
        let mut variant = None;
        let mut weight = None;
        let size;
        loop {
            // Special-case 'normal' because it is valid in each of
            // font-style, font-weight and font-variant.
            // Leaves the values to None, 'normal' is the initial value for each of them.
            if input.try(|input| input.expect_ident_matching("normal")).is_ok() {
                nb_normals += 1;
                continue;
            }
            if style.is_none() {
                if let Ok(value) = input.try(|input| font_style::parse(context, input)) {
                    style = Some(value);
                    continue
                }
            }
            if weight.is_none() {
                if let Ok(value) = input.try(|input| font_weight::parse(context, input)) {
                    weight = Some(value);
                    continue
                }
            }
            if variant.is_none() {
                if let Ok(value) = input.try(|input| font_variant::parse(context, input)) {
                    variant = Some(value);
                    continue
                }
            }
            size = Some(try!(font_size::parse(context, input)));
            break
        }
        #[inline]
        fn count<T>(opt: &Option<T>) -> u8 {
            if opt.is_some() { 1 } else { 0 }
        }
        if size.is_none() || (count(&style) + count(&weight) + count(&variant) + nb_normals) > 3 {
            return Err(())
        }
        let line_height = if input.try(|input| input.expect_delim('/')).is_ok() {
            Some(try!(line_height::parse(context, input)))
        } else {
            None
        };
        let family = try!(input.parse_comma_separated(font_family::parse_one_family));
        Ok(Longhands {
            font_style: style,
            font_variant: variant,
            font_weight: weight,
            font_size: size,
            line_height: line_height,
            font_family: Some(font_family::SpecifiedValue(family))
        })
    }

    // This may be a bit off, unsure, possibly needs changes
    pub fn serialize<'a, W, I>(dest: &mut W, declarations: I) -> fmt::Result
        where W: fmt::Write, I: Iterator<Item=&'a PropertyDeclaration> {

        let mut font_family = None;
        let mut font_style = None;
        let mut font_variant = None;
        let mut font_weight = None;
        let mut font_size = None;
        let mut font_stretch = None;
        let mut line_height = None;

        for decl in declarations {
            match *decl {
                PropertyDeclaration::FontFamily(ref value) => { font_family = Some(value); },
                PropertyDeclaration::FontStyle(ref value) => { font_style = Some(value); },
                PropertyDeclaration::FontVariant(ref value) => { font_variant = Some(value); },
                PropertyDeclaration::FontWeight(ref value) => { font_weight = Some(value); },
                PropertyDeclaration::FontSize(ref value) => { font_size = Some(value); },
                PropertyDeclaration::FontStretch(ref value) => { font_stretch = Some(value); },
                PropertyDeclaration::LineHeight(ref value) => { line_height = Some(value); },
                _ => return Err(fmt::Error)
            }
        }

        let (font_family, font_style, font_variant, font_weight, font_size, font_stretch, line_height) =
            try_unwrap_longhands!(
                font_family, font_style, font_variant, font_weight, font_size, font_stretch, line_height
            );

        if let DeclaredValue::Value(ref style) = *font_style {
            try!(style.to_css(dest));
            try!(write!(dest, " "));
        }

        if let DeclaredValue::Value(ref variant) = *font_variant {
            try!(variant.to_css(dest));
            try!(write!(dest, " "));
        }

        if let DeclaredValue::Value(ref weight) = *font_weight {
            try!(weight.to_css(dest));
            try!(write!(dest, " "));
        }

        if let DeclaredValue::Value(ref stretch) = *font_stretch {
            try!(stretch.to_css(dest));
            try!(write!(dest, " "));
        }

        try!(font_size.to_css(dest));
        if let DeclaredValue::Value(ref height) = *line_height {
            match *height {
                line_height::SpecifiedValue::Normal => {},
                _ => {
                    try!(write!(dest, "/"));
                    try!(height.to_css(dest));
                }
            }
        }

        try!(write!(dest, " "));

        font_family.to_css(dest)
    }
</%helpers:shorthand>
