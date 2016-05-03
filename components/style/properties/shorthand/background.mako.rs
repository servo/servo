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
</%helpers:shorthand>

