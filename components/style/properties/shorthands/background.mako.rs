/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

<%namespace name="helpers" file="/helpers.mako.rs" />

// TODO: other background-* properties
<%helpers:shorthand name="background"
                    sub_properties="background-color background-position-x background-position-y background-repeat
                                    background-attachment background-image background-size background-origin
                                    background-clip"
                    spec="https://drafts.csswg.org/css-backgrounds/#the-background">
    use properties::longhands::{background_position_x, background_position_y, background_repeat};
    use properties::longhands::{background_attachment, background_image, background_size, background_origin};
    use properties::longhands::background_clip;
    use properties::longhands::background_clip::single_value::computed_value::T as Clip;
    use properties::longhands::background_origin::single_value::computed_value::T as Origin;
    use values::specified::{Color, Position, PositionComponent};
    use parser::Parse;

    // FIXME(emilio): Should be the same type!
    impl From<background_origin::single_value::SpecifiedValue> for background_clip::single_value::SpecifiedValue {
        fn from(origin: background_origin::single_value::SpecifiedValue) ->
            background_clip::single_value::SpecifiedValue {
            match origin {
                background_origin::single_value::SpecifiedValue::ContentBox =>
                    background_clip::single_value::SpecifiedValue::ContentBox,
                background_origin::single_value::SpecifiedValue::PaddingBox =>
                    background_clip::single_value::SpecifiedValue::PaddingBox,
                background_origin::single_value::SpecifiedValue::BorderBox =>
                    background_clip::single_value::SpecifiedValue::BorderBox,
            }
        }
    }

    pub fn parse_value<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Longhands, ParseError<'i>> {
        let mut background_color = None;

        % for name in "image position_x position_y repeat size attachment origin clip".split():
            // Vec grows from 0 to 4 by default on first push().  So allocate
            // with capacity 1, so in the common case of only one item we don't
            // way overallocate.  Note that we always push at least one item if
            // parsing succeeds.
            let mut background_${name} = background_${name}::SpecifiedValue(Vec::with_capacity(1));
        % endfor
        input.parse_comma_separated(|input| {
            // background-color can only be in the last element, so if it
            // is parsed anywhere before, the value is invalid.
            if background_color.is_some() {
                return Err(input.new_custom_error(StyleParseErrorKind::UnspecifiedError));
            }

            % for name in "image position repeat size attachment origin clip".split():
                let mut ${name} = None;
            % endfor
            loop {
                if background_color.is_none() {
                    if let Ok(value) = input.try(|i| Color::parse(context, i)) {
                        background_color = Some(value);
                        continue
                    }
                }
                if position.is_none() {
                    if let Ok(value) = input.try(|input| Position::parse(context, input)) {
                        position = Some(value);

                        // Parse background size, if applicable.
                        size = input.try(|input| {
                            input.expect_delim('/')?;
                            background_size::single_value::parse(context, input)
                        }).ok();

                        continue
                    }
                }
                % for name in "image repeat attachment origin clip".split():
                    if ${name}.is_none() {
                        if let Ok(value) = input.try(|input| background_${name}::single_value
                                                                               ::parse(context, input)) {
                            ${name} = Some(value);
                            continue
                        }
                    }
                % endfor
                break
            }
            if clip.is_none() {
                if let Some(origin) = origin {
                    clip = Some(background_clip::single_value::SpecifiedValue::from(origin));
                }
            }
            let mut any = false;
            % for name in "image position repeat size attachment origin clip".split():
                any = any || ${name}.is_some();
            % endfor
            any = any || background_color.is_some();
            if any {
                if let Some(position) = position {
                    background_position_x.0.push(position.horizontal);
                    background_position_y.0.push(position.vertical);
                } else {
                    background_position_x.0.push(PositionComponent::zero());
                    background_position_y.0.push(PositionComponent::zero());
                }
                % for name in "image repeat size attachment origin clip".split():
                    if let Some(bg_${name}) = ${name} {
                        background_${name}.0.push(bg_${name});
                    } else {
                        background_${name}.0.push(background_${name}::single_value
                                                                    ::get_initial_specified_value());
                    }
                % endfor
                Ok(())
            } else {
                Err(input.new_custom_error(StyleParseErrorKind::UnspecifiedError))
            }
        })?;

        Ok(expanded! {
             background_color: background_color.unwrap_or(Color::transparent()),
             background_image: background_image,
             background_position_x: background_position_x,
             background_position_y: background_position_y,
             background_repeat: background_repeat,
             background_attachment: background_attachment,
             background_size: background_size,
             background_origin: background_origin,
             background_clip: background_clip,
         })
    }

    impl<'a> ToCss for LonghandsToSerialize<'a>  {
        fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result where W: fmt::Write {
            let len = self.background_image.0.len();
            // There should be at least one declared value
            if len == 0 {
                return Ok(());
            }

            // If a value list length is differs then we don't do a shorthand serialization.
            // The exceptions to this is color which appears once only and is serialized
            // with the last item.
            % for name in "image position_x position_y size repeat origin clip attachment".split():
                if len != self.background_${name}.0.len() {
                    return Ok(());
                }
            % endfor

            for i in 0..len {
                % for name in "image position_x position_y repeat size attachment origin clip".split():
                    let ${name} = &self.background_${name}.0[i];
                % endfor

                if i != 0 {
                    dest.write_str(", ")?;
                }

                if i == len - 1 {
                    self.background_color.to_css(dest)?;
                    dest.write_str(" ")?;
                }

                image.to_css(dest)?;
                % for name in "repeat attachment".split():
                    dest.write_str(" ")?;
                    ${name}.to_css(dest)?;
                % endfor

                dest.write_str(" ")?;
                Position {
                    horizontal: position_x.clone(),
                    vertical: position_y.clone()
                }.to_css(dest)?;

                if *size != background_size::single_value::get_initial_specified_value() {
                    dest.write_str(" / ")?;
                    size.to_css(dest)?;
                }

                if *origin != Origin::PaddingBox || *clip != Clip::BorderBox {
                    dest.write_str(" ")?;
                    origin.to_css(dest)?;
                    if *clip != From::from(*origin) {
                        dest.write_str(" ")?;
                        clip.to_css(dest)?;
                    }
                }
            }

            Ok(())
        }
    }
</%helpers:shorthand>

<%helpers:shorthand name="background-position"
                    flags="SHORTHAND_IN_GETCS"
                    sub_properties="background-position-x background-position-y"
                    spec="https://drafts.csswg.org/css-backgrounds-4/#the-background-position">
    use properties::longhands::{background_position_x, background_position_y};
    use values::specified::AllowQuirks;
    use values::specified::position::Position;

    pub fn parse_value<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Longhands, ParseError<'i>> {
        // Vec grows from 0 to 4 by default on first push().  So allocate with
        // capacity 1, so in the common case of only one item we don't way
        // overallocate.  Note that we always push at least one item if parsing
        // succeeds.
        let mut position_x = background_position_x::SpecifiedValue(Vec::with_capacity(1));
        let mut position_y = background_position_y::SpecifiedValue(Vec::with_capacity(1));
        let mut any = false;

        input.parse_comma_separated(|input| {
            let value = Position::parse_quirky(context, input, AllowQuirks::Yes)?;
            position_x.0.push(value.horizontal);
            position_y.0.push(value.vertical);
            any = true;
            Ok(())
        })?;
        if !any {
            return Err(input.new_custom_error(StyleParseErrorKind::UnspecifiedError));
        }

        Ok(expanded! {
            background_position_x: position_x,
            background_position_y: position_y,
        })
    }

    impl<'a> ToCss for LonghandsToSerialize<'a>  {
        fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result where W: fmt::Write {
            let len = self.background_position_x.0.len();
            if len == 0 || len != self.background_position_y.0.len() {
                return Ok(());
            }
            for i in 0..len {
                Position {
                    horizontal: self.background_position_x.0[i].clone(),
                    vertical: self.background_position_y.0[i].clone()
                }.to_css(dest)?;

                if i < len - 1 {
                    dest.write_str(", ")?;
                }
            }
            Ok(())
        }
    }
</%helpers:shorthand>
