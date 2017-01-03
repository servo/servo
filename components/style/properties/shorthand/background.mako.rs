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
    use properties::longhands::{background_color, background_position_x, background_position_y, background_repeat};
    use properties::longhands::{background_attachment, background_image, background_size, background_origin};
    use properties::longhands::background_clip;
    use values::specified::position::Position;
    use parser::Parse;

    impl From<background_origin::single_value::SpecifiedValue> for background_clip::single_value::SpecifiedValue {
        fn from(origin: background_origin::single_value::SpecifiedValue) ->
            background_clip::single_value::SpecifiedValue {
            match origin {
                background_origin::single_value::SpecifiedValue::content_box =>
                    background_clip::single_value::SpecifiedValue::content_box,
                background_origin::single_value::SpecifiedValue::padding_box =>
                    background_clip::single_value::SpecifiedValue::padding_box,
                background_origin::single_value::SpecifiedValue::border_box =>
                    background_clip::single_value::SpecifiedValue::border_box,
            }
        }
    }

    pub fn parse_value(context: &ParserContext, input: &mut Parser) -> Result<Longhands, ()> {
        let mut background_color = None;

        % for name in "image position_x position_y repeat size attachment origin clip".split():
            let mut background_${name} = background_${name}::SpecifiedValue(Vec::new());
        % endfor
        try!(input.parse_comma_separated(|input| {
            % for name in "image position_x position_y repeat size attachment origin clip".split():
                let mut ${name} = None;
            % endfor
            loop {
                if let Ok(value) = input.try(|input| background_color::parse(context, input)) {
                    if background_color.is_none() {
                        background_color = Some(value);
                        continue
                    } else {
                        // color can only be the last element
                        return Err(())
                    }
                }
                if position_x.is_none() && position_y.is_none() {
                    if let Ok(value) = input.try(|input| Position::parse(context, input)) {
                        position_x = Some(value.horizontal);
                        position_y = Some(value.vertical);

                        // Parse background size, if applicable.
                        size = input.try(|input| {
                            try!(input.expect_delim('/'));
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
            % for name in "image position_x position_y repeat size attachment origin clip".split():
                any = any || ${name}.is_some();
            % endfor
            any = any || background_color.is_some();
            if any {
                if position_x.is_some() || position_y.is_some() {
                    % for name in "position_x position_y".split():
                        if let Some(bg_${name}) = ${name} {
                            background_${name}.0.push(bg_${name});
                        }
                    % endfor
                } else {
                    % for name in "position_x position_y".split():
                        background_${name}.0.push(background_${name}::single_value
                                                                    ::get_initial_position_value());
                    % endfor
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
                Err(())
            }
        }));

        Ok(Longhands {
             background_color: background_color,
             background_image: Some(background_image),
             background_position_x: Some(background_position_x),
             background_position_y: Some(background_position_y),
             background_repeat: Some(background_repeat),
             background_attachment: Some(background_attachment),
             background_size: Some(background_size),
             background_origin: Some(background_origin),
             background_clip: Some(background_clip),
         })
    }

    impl<'a> LonghandsToSerialize<'a>  {
        fn to_css_declared<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            // mako doesn't like ampersands following `<`
            fn extract_value<T>(x: &DeclaredValue<T>) -> Option< &T> {
                match *x {
                    DeclaredValue::Value(ref val) => Some(val),
                    _ => None,
                }
            }
            use std::cmp;
            let mut len = 0;
            % for name in "image position_x position_y repeat size attachment origin clip".split():
                len = cmp::max(len, extract_value(self.background_${name}).map(|i| i.0.len())
                                                                          .unwrap_or(0));
            % endfor

            // There should be at least one declared value
            if len == 0 {
                return dest.write_str("")
            }

            let mut first = true;
            for i in 0..len {
                % for name in "image position_x position_y repeat size attachment origin clip".split():
                    let ${name} = if let DeclaredValue::Value(ref arr) = *self.background_${name} {
                        arr.0.get(i % arr.0.len())
                    } else {
                        None
                    };
                % endfor
                let color = if i == len - 1 {
                    Some(self.background_color)
                } else {
                    None
                };

                if first {
                    first = false;
                } else {
                    try!(write!(dest, ", "));
                }
                match color {
                    Some(&DeclaredValue::Value(ref color)) => {
                        try!(color.to_css(dest));
                        try!(write!(dest, " "));
                    },
                    Some(_) => {
                        try!(write!(dest, "transparent "));
                    }
                    // Not yet the last one
                    None => ()
                };


                if let Some(image) = image {
                    try!(image.to_css(dest));
                } else {
                    try!(write!(dest, "none"));
                }

                try!(write!(dest, " "));

                if let Some(repeat) = repeat {
                    try!(repeat.to_css(dest));
                } else {
                    try!(write!(dest, "repeat"));
                }

                try!(write!(dest, " "));

                if let Some(attachment) = attachment {
                    try!(attachment.to_css(dest));
                } else {
                    try!(write!(dest, "scroll"));
                }

                try!(write!(dest, " "));

                try!(position_x.unwrap_or(&background_position_x::single_value
                                                                ::get_initial_position_value())
                             .to_css(dest));

                try!(write!(dest, " "));

                try!(position_y.unwrap_or(&background_position_y::single_value
                                                                ::get_initial_position_value())
                             .to_css(dest));

                if let Some(size) = size {
                    try!(write!(dest, " / "));
                    try!(size.to_css(dest));
                }

                match (origin, clip) {
                    (Some(origin), Some(clip)) => {
                        use properties::longhands::background_origin::single_value::computed_value::T as Origin;
                        use properties::longhands::background_clip::single_value::computed_value::T as Clip;

                        try!(write!(dest, " "));

                        match (origin, clip) {
                            (&Origin::padding_box, &Clip::padding_box) => {
                                try!(origin.to_css(dest));
                            },
                            (&Origin::border_box, &Clip::border_box) => {
                                try!(origin.to_css(dest));
                            },
                            (&Origin::content_box, &Clip::content_box) => {
                                try!(origin.to_css(dest));
                            },
                            _ => {
                                try!(origin.to_css(dest));
                                try!(write!(dest, " "));
                                try!(clip.to_css(dest));
                            }
                        }
                    },
                    _ => {}
                };
            }


            Ok(())
        }
    }
</%helpers:shorthand>

<%helpers:shorthand name="background-position"
                    sub_properties="background-position-x background-position-y"
                    spec="https://drafts.csswg.org/css-backgrounds-4/#the-background-position">
    use properties::longhands::{background_position_x,background_position_y};
    use values::specified::position::Position;
    use parser::Parse;

    pub fn parse_value(context: &ParserContext, input: &mut Parser) -> Result<Longhands, ()> {
        let mut position_x = background_position_x::SpecifiedValue(Vec::new());
        let mut position_y = background_position_y::SpecifiedValue(Vec::new());
        let mut any = false;

        try!(input.parse_comma_separated(|input| {
            loop {
                if let Ok(value) = input.try(|input| Position::parse(context, input)) {
                    position_x.0.push(value.horizontal);
                    position_y.0.push(value.vertical);
                    any = true;
                    continue
                }
                break
            }
            Ok(())
        }));
        if any == false {
            return Err(());
        }

        Ok(Longhands {
            background_position_x: Some(position_x),
            background_position_y: Some(position_y),
        })
    }

    impl<'a> LonghandsToSerialize<'a>  {
        fn to_css_declared<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            // mako doesn't like ampersands following `<`
            fn extract_value<T>(x: &DeclaredValue<T>) -> Option< &T> {
                match *x {
                    DeclaredValue::Value(ref val) => Some(val),
                    _ => None,
                }
            }
            use std::cmp;
            let mut len = 0;
            % for name in "x y".split():
                len = cmp::max(len, extract_value(self.background_position_${name})
                                                      .map(|i| i.0.len())
                                                      .unwrap_or(0));
            % endfor

            // There should be at least one declared value
            if len == 0 {
                return dest.write_str("")
            }

            for i in 0..len {
                % for name in "x y".split():
                    let position_${name} = if let DeclaredValue::Value(ref arr) =
                                           *self.background_position_${name} {
                        arr.0.get(i % arr.0.len())
                    } else {
                        None
                    };
                % endfor

                try!(position_x.unwrap_or(&background_position_x::single_value
                                                                ::get_initial_position_value())
                               .to_css(dest));

                try!(write!(dest, " "));

                try!(position_y.unwrap_or(&background_position_y::single_value
                                                                ::get_initial_position_value())
                               .to_css(dest));
            }

            Ok(())
        }
    }
</%helpers:shorthand>
