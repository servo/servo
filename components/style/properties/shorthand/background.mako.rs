/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

<%namespace name="helpers" file="/helpers.mako.rs" />

// TODO: other background-* properties
<%helpers:shorthand name="background"
                    sub_properties="background-color background-position background-repeat background-attachment
                                    background-image background-size background-origin background-clip">
    use properties::longhands::{background_color, background_position, background_repeat, background_attachment};
    use properties::longhands::{background_image, background_size, background_origin, background_clip};

    pub fn parse_value(context: &ParserContext, input: &mut Parser) -> Result<Longhands, ()> {
        % for name in "image position repeat size attachment origin clip".split():
            let mut any_${name} = false;
        % endfor

        let mut background_color = None;
        let vec = try!(input.parse_comma_separated(|input| {
            % for name in "image position repeat size attachment origin clip".split():
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
                if position.is_none() {
                    if let Ok(value) = input.try(|input| background_position::single_value::parse(context, input)) {
                        position = Some(value);
                        any_position = true;

                        // Parse background size, if applicable.
                        size = input.try(|input| {
                            try!(input.expect_delim('/'));
                            background_size::single_value::parse(context, input)
                        }).ok();
                        if let Some(_) = size {
                            any_size = true;
                        }

                        continue
                    }
                }
                % for name in "image repeat attachment origin clip".split():
                    if ${name}.is_none() {
                        if let Ok(value) = input.try(|input| background_${name}::single_value::parse(context, input)) {
                            ${name} = Some(value);
                            any_${name} = true;
                            continue
                        }
                    }
                % endfor
                break
            }
            Ok((image, position, repeat, size, attachment, origin, clip))
        }));

        if !(any_image || any_position || any_repeat || any_size ||
             any_attachment || any_origin || any_clip ||
             background_color.is_some()) {
            return Err(())
        }

        % for name in "image position repeat size attachment origin clip".split():
            let mut background_${name} = if any_${name} {
                Some(background_${name}::SpecifiedValue(Vec::new()))
            } else {
                None
            };
        % endfor

        for i in vec {
            % for i,name in enumerate("image position repeat size attachment origin clip".split()):
                if let Some(ref mut buf) = background_${name} {
                    if let Some(bg_${name}) = i.${i} {
                        buf.0.push(bg_${name})
                    } else {
                        buf.0.push(background_${name}::single_value::get_initial_specified_value())
                    }
                }
            % endfor
        }

        Ok(Longhands {
             background_color: background_color,
             background_image: background_image,
             background_position: background_position,
             background_repeat: background_repeat,
             background_attachment: background_attachment,
             background_size: background_size,
             background_origin: background_origin,
             background_clip: background_clip,
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
            use std::iter::{once, repeat};
            let mut len = 0;
            % for name in "image position repeat size attachment origin clip".split():
                len = cmp::max(len, extract_value(self.background_${name}).map(|i| i.0.len())
                                                                          .unwrap_or(0));
            % endfor

            // There should be at least one declared value
            if len == 0 {
                return dest.write_str("")
            }

            let iter = repeat(None).take(len - 1).chain(once(Some(self.background_color)))
                % for name in "image position repeat size attachment origin clip".split():
                    .zip(extract_value(self.background_${name}).into_iter()
                                                               .flat_map(|x| x.0.iter())
                                                               .map(Some).chain(repeat(None)))
                % endfor
                ;
            let mut first = true;
            for (((((((color, image), position), repeat), size), attachment), origin), clip) in iter {
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
                        try!(write!(dest, " "));
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

                try!(position.unwrap_or(&background_position::single_value
                                                            ::get_initial_specified_value())
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
                    (Some(origin), _) => {
                        try!(write!(dest, " "));
                        try!(origin.to_css(dest));
                    },
                    (_, Some(clip)) => {
                        try!(write!(dest, " "));
                        try!(clip.to_css(dest));
                    },
                    _ => {}
                };
            }


            Ok(())
        }
    }
</%helpers:shorthand>
