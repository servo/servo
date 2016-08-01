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
        let mut color = None;
        let mut image = None;
        let mut position = None;
        let mut repeat = None;
        let mut size = None;
        let mut attachment = None;
        let mut any = false;
        let mut origin = None;
        let mut clip = None;

        loop {
            if position.is_none() {
                if let Ok(value) = input.try(|input| background_position::parse(context, input)) {
                    position = Some(value);
                    any = true;

                    // Parse background size, if applicable.
                    size = input.try(|input| {
                        try!(input.expect_delim('/'));
                        background_size::parse(context, input)
                    }).ok();

                    continue
                }
            }
            if color.is_none() {
                if let Ok(value) = input.try(|input| background_color::parse(context, input)) {
                    color = Some(value);
                    any = true;
                    continue
                }
            }
            if image.is_none() {
                if let Ok(value) = input.try(|input| background_image::parse(context, input)) {
                    image = Some(value);
                    any = true;
                    continue
                }
            }
            if repeat.is_none() {
                if let Ok(value) = input.try(|input| background_repeat::parse(context, input)) {
                    repeat = Some(value);
                    any = true;
                    continue
                }
            }
            if attachment.is_none() {
                if let Ok(value) = input.try(|input| background_attachment::parse(context, input)) {
                    attachment = Some(value);
                    any = true;
                    continue
                }
            }
            if origin.is_none() {
                if let Ok(value) = input.try(|input| background_origin::parse(context, input)) {
                    origin = Some(value);
                    any = true;
                    continue
                }
            }
            if clip.is_none() {
                if let Ok(value) = input.try(|input| background_clip::parse(context, input)) {
                    clip = Some(value);
                    any = true;
                    continue
                }
            }
            break
        }

        if any {
            Ok(Longhands {
                background_color: color,
                background_image: image,
                background_position: position,
                background_repeat: repeat,
                background_attachment: attachment,
                background_size: size,
                background_origin: origin,
                background_clip: clip,
            })
        } else {
            Err(())
        }
    }

    pub fn serialize<'a, W, I>(dest: &mut W, declarations: I) -> fmt::Result
        where W: fmt::Write, I: Iterator<Item=&'a PropertyDeclaration> {

        let mut color = None;
        let mut position = None;
        let mut repeat = None;
        let mut attachment = None;
        let mut image = None;
        let mut size = None;
        let mut origin = None;
        let mut clip = None;

        for decl in declarations {
            match *decl {
                PropertyDeclaration::BackgroundColor(ref value) => { color = Some(value); },
                PropertyDeclaration::BackgroundPosition(ref value) => { position = Some(value); },
                PropertyDeclaration::BackgroundRepeat(ref value) => { repeat  = Some(value); },
                PropertyDeclaration::BackgroundAttachment(ref value) => { attachment = Some(value); },
                PropertyDeclaration::BackgroundImage(ref value) => { image = Some(value); },
                PropertyDeclaration::BackgroundSize(ref value) => { size = Some(value); },
                PropertyDeclaration::BackgroundOrigin(ref value) => { origin = Some(value); },
                PropertyDeclaration::BackgroundClip(ref value) => { clip = Some(value); },
            _ => return Err(fmt::Error)
            }
        }

        let (
            color, position, repeat, attachment,
            image, size, origin, clip
        ) =
        try_unwrap_longhands!(
            color, position, repeat, attachment,
            image, size, origin, clip
        );


        match *color {
            DeclaredValue::Value(ref color) => {
                try!(color.to_css(dest));
            },
            _ => {
                try!(write!(dest, "transparent"));
            }
        };

        try!(write!(dest, " "));

        match *image {
            DeclaredValue::Value(ref image) => {
                try!(image.to_css(dest));
            },
            _ => {
                try!(write!(dest, "none"));
            }
        };

        try!(write!(dest, " "));

        try!(repeat.to_css(dest));

        try!(write!(dest, " "));

        match *attachment {
            DeclaredValue::Value(ref attachment) => {
                try!(attachment.to_css(dest));
            },
            _ => {
                try!(write!(dest, "scroll"));
            }
        };

        try!(write!(dest, " "));

        try!(position.to_css(dest));

        if let DeclaredValue::Value(ref size) = *size {
            try!(write!(dest, " / "));
            try!(size.to_css(dest));
        }

        match (origin, clip) {
            (&DeclaredValue::Value(ref origin), &DeclaredValue::Value(ref clip)) => {
                use properties::longhands::background_origin::computed_value::T as Origin;
                use properties::longhands::background_clip::computed_value::T as Clip;

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
            (&DeclaredValue::Value(ref origin), _) => {
                try!(write!(dest, " "));
                try!(origin.to_css(dest));
            },
            (_, &DeclaredValue::Value(ref clip)) => {
                try!(write!(dest, " "));
                try!(clip.to_css(dest));
            },
            _ => {}
        };


        Ok(())
    }
</%helpers:shorthand>
