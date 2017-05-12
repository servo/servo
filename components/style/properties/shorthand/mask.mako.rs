/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

<%namespace name="helpers" file="/helpers.mako.rs" />

<%helpers:shorthand name="mask" products="gecko" extra_prefixes="webkit"
                    sub_properties="mask-mode mask-repeat mask-clip mask-origin mask-composite mask-position-x
                                    mask-position-y mask-size mask-image"
                    spec="https://drafts.fxtf.org/css-masking/#propdef-mask">
    use properties::longhands::{mask_mode, mask_repeat, mask_clip, mask_origin, mask_composite, mask_position_x,
                                mask_position_y};
    use properties::longhands::{mask_size, mask_image};
    use values::specified::{Position, PositionComponent};
    use parser::Parse;

    impl From<mask_origin::single_value::SpecifiedValue> for mask_clip::single_value::SpecifiedValue {
        fn from(origin: mask_origin::single_value::SpecifiedValue) -> mask_clip::single_value::SpecifiedValue {
            match origin {
                mask_origin::single_value::SpecifiedValue::content_box =>
                    mask_clip::single_value::SpecifiedValue::content_box,
                mask_origin::single_value::SpecifiedValue::padding_box =>
                    mask_clip::single_value::SpecifiedValue::padding_box,
                mask_origin::single_value::SpecifiedValue::border_box =>
                    mask_clip::single_value::SpecifiedValue::border_box,
                % if product == "gecko":
                mask_origin::single_value::SpecifiedValue::fill_box =>
                    mask_clip::single_value::SpecifiedValue::fill_box,
                mask_origin::single_value::SpecifiedValue::stroke_box =>
                    mask_clip::single_value::SpecifiedValue::stroke_box,
                mask_origin::single_value::SpecifiedValue::view_box =>
                    mask_clip::single_value::SpecifiedValue::view_box,
                % endif
            }
        }
    }

    pub fn parse_value(context: &ParserContext, input: &mut Parser) -> Result<Longhands, ()> {
        % for name in "image mode position_x position_y size repeat origin clip composite".split():
            let mut mask_${name} = mask_${name}::SpecifiedValue(Vec::new());
        % endfor

        try!(input.parse_comma_separated(|input| {
            % for name in "image mode position size repeat origin clip composite".split():
                let mut ${name} = None;
            % endfor
            loop {
                if image.is_none() {
                    if let Ok(value) = input.try(|input| mask_image::single_value
                                                                   ::parse(context, input)) {
                        image = Some(value);
                        continue
                    }
                }
                if position.is_none() {
                    if let Ok(value) = input.try(|input| Position::parse(context, input)) {
                        position = Some(value);

                        // Parse mask size, if applicable.
                        size = input.try(|input| {
                            try!(input.expect_delim('/'));
                            mask_size::single_value::parse(context, input)
                        }).ok();

                        continue
                    }
                }
                % for name in "repeat origin clip composite mode".split():
                    if ${name}.is_none() {
                        if let Ok(value) = input.try(|input| mask_${name}::single_value
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
                    clip = Some(mask_clip::single_value::SpecifiedValue::from(origin));
                }
            }
            let mut any = false;
            % for name in "image mode position size repeat origin clip composite".split():
                any = any || ${name}.is_some();
            % endfor
            if any {
                if let Some(position) = position {
                    mask_position_x.0.push(position.horizontal);
                    mask_position_y.0.push(position.vertical);
                } else {
                    mask_position_x.0.push(PositionComponent::zero());
                    mask_position_y.0.push(PositionComponent::zero());
                }
                % for name in "image mode size repeat origin clip composite".split():
                    if let Some(m_${name}) = ${name} {
                        mask_${name}.0.push(m_${name});
                    } else {
                        mask_${name}.0.push(mask_${name}::single_value
                                                        ::get_initial_specified_value());
                    }
                % endfor
                Ok(())
            } else {
                Err(())
            }
        }));

        Ok(Longhands {
            % for name in "image mode position_x position_y size repeat origin clip composite".split():
                mask_${name}: mask_${name},
            % endfor
         })
    }

    impl<'a> ToCss for LonghandsToSerialize<'a>  {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            use properties::longhands::mask_origin::single_value::computed_value::T as Origin;
            use properties::longhands::mask_clip::single_value::computed_value::T as Clip;

            let len = self.mask_image.0.len();
            if len == 0 {
                return Ok(());
            }
            % for name in "mode position_x position_y size repeat origin clip composite".split():
                if self.mask_${name}.0.len() != len {
                    return Ok(());
                }
            % endfor

            for i in 0..len {
                if i > 0 {
                    dest.write_str(", ")?;
                }

                % for name in "image mode position_x position_y size repeat origin clip composite".split():
                    let ${name} = &self.mask_${name}.0[i];
                % endfor

                image.to_css(dest)?;
                dest.write_str(" ")?;
                mode.to_css(dest)?;
                dest.write_str(" ")?;

                Position {
                    horizontal: position_x.clone(),
                    vertical: position_y.clone()
                }.to_css(dest)?;

                if *size != mask_size::single_value::get_initial_specified_value() {
                    dest.write_str(" / ")?;
                    size.to_css(dest)?;
                }
                dest.write_str(" ")?;
                repeat.to_css(dest)?;

                if *origin != Origin::border_box || *clip != Clip::border_box {
                    dest.write_str(" ")?;
                    origin.to_css(dest)?;
                    if *clip != From::from(*origin) {
                        dest.write_str(" ")?;
                        clip.to_css(dest)?;
                    }
                }

                dest.write_str(" ")?;
                composite.to_css(dest)?;
            }

            Ok(())
        }
    }
</%helpers:shorthand>

<%helpers:shorthand name="mask-position" products="gecko" extra_prefixes="webkit"
                    sub_properties="mask-position-x mask-position-y"
                    spec="https://drafts.csswg.org/css-masks-4/#the-mask-position">
    use properties::longhands::{mask_position_x,mask_position_y};
    use values::specified::position::Position;
    use parser::Parse;

    pub fn parse_value(context: &ParserContext, input: &mut Parser) -> Result<Longhands, ()> {
        let mut position_x = mask_position_x::SpecifiedValue(Vec::new());
        let mut position_y = mask_position_y::SpecifiedValue(Vec::new());
        let mut any = false;

        input.parse_comma_separated(|input| {
            let value = Position::parse(context, input)?;
            position_x.0.push(value.horizontal);
            position_y.0.push(value.vertical);
            any = true;
            Ok(())
        })?;
        if any == false {
            return Err(());
        }

        Ok(Longhands {
            mask_position_x: position_x,
            mask_position_y: position_y,
        })
    }

    impl<'a> ToCss for LonghandsToSerialize<'a>  {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            let len = self.mask_position_x.0.len();
            if len == 0 || self.mask_position_y.0.len() != len {
                return Ok(());
            }

            for i in 0..len {
                Position {
                    horizontal: self.mask_position_x.0[i].clone(),
                    vertical: self.mask_position_y.0[i].clone()
                }.to_css(dest)?;

                if i < len - 1 {
                    dest.write_str(", ")?;
                }
            }

            Ok(())
        }
    }
</%helpers:shorthand>
