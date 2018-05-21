/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

<%namespace name="helpers" file="/helpers.mako.rs" />

<%helpers:shorthand name="list-style"
                    sub_properties="list-style-position list-style-image list-style-type"
                    derive_serialize="True"
                    spec="https://drafts.csswg.org/css-lists/#propdef-list-style">
    use properties::longhands::{list_style_image, list_style_position, list_style_type};
    use values::specified::url::ImageUrlOrNone;

    pub fn parse_value<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Longhands, ParseError<'i>> {
        // `none` is ambiguous until we've finished parsing the shorthands, so we count the number
        // of times we see it.
        let mut nones = 0u8;
        let (mut image, mut position, mut list_style_type, mut any) = (None, None, None, false);
        loop {
            if input.try(|input| input.expect_ident_matching("none")).is_ok() {
                nones = nones + 1;
                if nones > 2 {
                    return Err(input.new_custom_error(SelectorParseErrorKind::UnexpectedIdent("none".into())))
                }
                any = true;
                continue
            }

            if image.is_none() {
                if let Ok(value) = input.try(|input| list_style_image::parse(context, input)) {
                    image = Some(value);
                    any = true;
                    continue
                }
            }

            if position.is_none() {
                if let Ok(value) = input.try(|input| list_style_position::parse(context, input)) {
                    position = Some(value);
                    any = true;
                    continue
                }
            }

            // list-style-type must be checked the last, because it accepts
            // arbitrary identifier for custom counter style, and thus may
            // affect values of list-style-position.
            if list_style_type.is_none() {
                if let Ok(value) = input.try(|input| list_style_type::parse(context, input)) {
                    list_style_type = Some(value);
                    any = true;
                    continue
                }
            }
            break
        }

        let position = unwrap_or_initial!(list_style_position, position);

        fn list_style_type_none() -> list_style_type::SpecifiedValue {
            % if product == "servo":
            list_style_type::SpecifiedValue::None
            % else:
            use values::generics::CounterStyleOrNone;
            list_style_type::SpecifiedValue::CounterStyle(CounterStyleOrNone::None)
            % endif
        }

        // If there are two `none`s, then we can't have a type or image; if there is one `none`,
        // then we can't have both a type *and* an image; if there is no `none` then we're fine as
        // long as we parsed something.
        match (any, nones, list_style_type, image) {
            (true, 2, None, None) => {
                Ok(expanded! {
                    list_style_position: position,
                    list_style_image: ImageUrlOrNone::none(),
                    list_style_type: list_style_type_none(),
                })
            }
            (true, 1, None, Some(image)) => {
                Ok(expanded! {
                    list_style_position: position,
                    list_style_image: image,
                    list_style_type: list_style_type_none(),
                })
            }
            (true, 1, Some(list_style_type), None) => {
                Ok(expanded! {
                    list_style_position: position,
                    list_style_image: ImageUrlOrNone::none(),
                    list_style_type: list_style_type,
                })
            }
            (true, 1, None, None) => {
                Ok(expanded! {
                    list_style_position: position,
                    list_style_image: ImageUrlOrNone::none(),
                    list_style_type: list_style_type_none(),
                })
            }
            (true, 0, list_style_type, image) => {
                Ok(expanded! {
                    list_style_position: position,
                    list_style_image: unwrap_or_initial!(list_style_image, image),
                    list_style_type: unwrap_or_initial!(list_style_type),
                })
            }
            _ => Err(input.new_custom_error(StyleParseErrorKind::UnspecifiedError)),
        }
    }
</%helpers:shorthand>
