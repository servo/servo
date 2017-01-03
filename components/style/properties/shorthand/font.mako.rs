/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

<%namespace name="helpers" file="/helpers.mako.rs" />

<%helpers:shorthand name="font" sub_properties="font-style font-variant font-weight
                                                font-size line-height font-family"
                    spec="https://drafts.csswg.org/css-fonts-3/#propdef-font">
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
    impl<'a> LonghandsToSerialize<'a>  {
        fn to_css_declared<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            if let DeclaredValue::Value(ref style) = *self.font_style {
                try!(style.to_css(dest));
                try!(write!(dest, " "));
            }

            if let DeclaredValue::Value(ref variant) = *self.font_variant {
                try!(variant.to_css(dest));
                try!(write!(dest, " "));
            }

            if let DeclaredValue::Value(ref weight) = *self.font_weight {
                try!(weight.to_css(dest));
                try!(write!(dest, " "));
            }

            try!(self.font_size.to_css(dest));
            if let DeclaredValue::Value(ref height) = *self.line_height {
                match *height {
                    line_height::SpecifiedValue::Normal => {},
                    _ => {
                        try!(write!(dest, "/"));
                        try!(height.to_css(dest));
                    }
                }
            }

            try!(write!(dest, " "));

            self.font_family.to_css(dest)
        }
    }
</%helpers:shorthand>
