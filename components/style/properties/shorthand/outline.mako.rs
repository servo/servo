/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

<%namespace name="helpers" file="/helpers.mako.rs" />

<%helpers:shorthand name="outline" sub_properties="outline-color outline-style outline-width">
    use properties::longhands::outline_width;
    use values::specified;

    let _unused = context;
    let mut color = None;
    let mut style = None;
    let mut width = None;
    let mut any = false;
    loop {
        if color.is_none() {
            if let Ok(value) = input.try(specified::CSSColor::parse) {
                color = Some(value);
                any = true;
                continue
            }
        }
        if style.is_none() {
            if let Ok(value) = input.try(specified::BorderStyle::parse) {
                style = Some(value);
                any = true;
                continue
            }
        }
        if width.is_none() {
            if let Ok(value) = input.try(|input| outline_width::parse(context, input)) {
                width = Some(value);
                any = true;
                continue
            }
        }
        break
    }
    if any {
        Ok(Longhands {
            outline_color: color,
            outline_style: style,
            outline_width: width,
        })
    } else {
        Err(())
    }
</%helpers:shorthand>

<%helpers:shorthand name="-moz-outline-radius" sub_properties="${' '.join(
    '-moz-outline-radius-%s' % corner
    for corner in ['topleft', 'topright', 'bottomright', 'bottomleft']
)}" products="gecko">
    use properties::shorthands;

    // Re-use border-radius parsing.
    shorthands::border_radius::parse_value(context, input).map(|longhands| {
        Longhands {
            _moz_outline_radius_topleft: longhands.border_top_left_radius,
            _moz_outline_radius_topright: longhands.border_top_right_radius,
            _moz_outline_radius_bottomright: longhands.border_bottom_right_radius,
            _moz_outline_radius_bottomleft: longhands.border_bottom_left_radius,
        }
    })
</%helpers:shorthand>
