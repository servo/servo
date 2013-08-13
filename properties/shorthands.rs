/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

pub use super::longhands::*;


macro_rules! shorthand(
    ($name: ident[$($longhand: ident),+] |input| $parser: expr) => {
        pub mod $name {
            use super::*;
            struct Longhands {
                $( $longhand: Option<$longhand::SpecifiedValue> ),+
            }
            fn parse(input: &[ComponentValue]) -> Option<Longhands> { $parser }
        }
    };
)


// The value of each longhand is the same as the value of the shorthand
macro_rules! duplicating_shorthand(
    ($name: ident,  $parser_function: expr, $($longhand: ident),+) => {
        shorthand!($name [$($longhand),+] |input| {
            do $parser_function(input).map_move |value| {
                Longhands { $( $longhand: Some(value) ),+ }
            }
        })
    };
)


macro_rules! four_side_shorthand(
    ($name: ident, $parser_function: expr,
     $top: ident, $right: ident, $bottom: ident, $left: ident) => {
        shorthand!($name [$top, $right, $bottom, $left] |input| {
            let mut iter = input.skip_whitespace().map($parser_function);
            // zero or more than four values is invalid.
            // one value sets them all
            // two values set (top, bottom) and (left, right)
            // three values set top, (left, right) and bottom
            // four values set them in order
            let top = iter.next().unwrap_or_default(None);
            let right = iter.next().unwrap_or_default(top);
            let bottom = iter.next().unwrap_or_default(top);
            let left = iter.next().unwrap_or_default(right);
            if top.is_some() && right.is_some() && bottom.is_some() && left.is_some()
            && iter.next().is_none() {
                Some(Longhands { $top: top, $right: right, $bottom: bottom, $left: left })
            } else {
                None
            }
        })
    };
)


// TODO: other background-* properties
shorthand!(background [
    background_color
] |input| {
    do one_component_value(input).chain(CSSColor::parse).map_move |color| {
        Longhands { background_color: Some(color) }
    }
})


duplicating_shorthand!(border_color, border_top_color::parse,
    border_top_color,
    border_right_color,
    border_bottom_color,
    border_left_color
)

duplicating_shorthand!(border_width, border_top_width::parse,
    border_top_width,
    border_right_width,
    border_bottom_width,
    border_left_width
)

duplicating_shorthand!(border_style, border_top_style::parse,
    border_top_style,
    border_right_style,
    border_bottom_style,
    border_left_style
)


pub fn parse_border(input: &[ComponentValue]) -> Option<(Option<CSSColor>, Option<BorderStyle>,
                                                         Option<specified::Length>)> {
    let mut color = None;
    let mut style = None;
    let mut width = None;
    let mut any = false;
    for component_value in input.skip_whitespace() {
        if color.is_none() {
            match CSSColor::parse(component_value) {
                Some(c) => { color = Some(c); any = true; loop },
                None => ()
            }
        }
        if style.is_none() {
            match BorderStyle::parse(component_value) {
                Some(s) => { style = Some(s); any = true; loop },
                None => ()
            }
        }
        if width.is_none() {
            match parse_border_width(component_value) {
                Some(w) => { width = Some(w); any = true; loop },
                None => ()
            }
        }
        return None
    }
    if any { Some((color, style, width)) } else { None }
}


shorthand!(border_top [
    border_top_color,
    border_top_width,
    border_top_style
] |input| {
    do parse_border(input).map_move |(color, style, width)| {
        Longhands { border_top_color: color, border_top_style: style,
                    border_top_width: width }
    }
})

shorthand!(border_right [
    border_right_color,
    border_right_width,
    border_right_style
] |input| {
    do parse_border(input).map_move |(color, style, width)| {
        Longhands { border_right_color: color, border_right_style: style,
                    border_right_width: width }
    }
})

shorthand!(border_bottom [
    border_bottom_color,
    border_bottom_width,
    border_bottom_style
] |input| {
    do parse_border(input).map_move |(color, style, width)| {
        Longhands { border_bottom_color: color, border_bottom_style: style,
                    border_bottom_width: width }
    }
})

shorthand!(border_left [
    border_left_color,
    border_left_width,
    border_left_style
] |input| {
    do parse_border(input).map_move |(color, style, width)| {
        Longhands { border_left_color: color, border_left_style: style,
                    border_left_width: width }
    }
})

shorthand!(border [
    border_top_color,
    border_top_width,
    border_top_style,
    border_right_color,
    border_right_width,
    border_right_style,
    border_bottom_color,
    border_bottom_width,
    border_bottom_style,
    border_left_color,
    border_left_width,
    border_left_style
] |input| {
    do parse_border(input).map_move |(color, style, width)| {
        Longhands {
            border_top_color: color, border_top_style: style, border_top_width: width,
            border_right_color: color, border_right_style: style, border_right_width: width,
            border_bottom_color: color, border_bottom_style: style, border_bottom_width: width,
            border_left_color: color, border_left_style: style, border_left_width: width,
        }
    }
})


// TODO: system fonts
shorthand!(font [
    font_style,
    font_variant,
    font_weight,
    font_size,
    line_height,
    font_family
] |input| {
    let mut iter = input.skip_whitespace();
    let mut nb_normals = 0u;
    let mut style = None;
    let mut variant = None;
    let mut weight = None;
    let mut size = None;
    let mut line_height = None;
    for component_value in iter {
        // Special-case 'normal' because it is valid in each of
        // font-style, font-weight and font-variant.
        // Leaves the values to None, 'normal' is the initial value for each of them.
        if get_ident_lower(component_value).filtered(
                |v| eq_ignore_ascii_case(v.as_slice(), "normal")).is_some() {
            nb_normals += 1;
            loop;
        }
        if style.is_none() {
            match font_style::from_component_value(component_value) {
                Some(s) => { style = Some(s); loop },
                None => ()
            }
        }
        if weight.is_none() {
            match font_weight::from_component_value(component_value) {
                Some(w) => { weight = Some(w); loop },
                None => ()
            }
        }
        if variant.is_none() {
            match font_variant::from_component_value(component_value) {
                Some(v) => { variant = Some(v); loop },
                None => ()
            }
        }
        match font_size::from_component_value(component_value) {
            Some(s) => { size = Some(s); break },
            None => return None
        }
    }
    #[inline]
    fn count<T>(opt: &Option<T>) -> uint {
        match opt {
            &Some(_) => 1,
            &None => 0,
        }
    }
    if size.is_none() || (count(&style) + count(&weight) + count(&variant) + nb_normals) > 3 {
        return None
    }
    let mut iter = iter.peekable();
    match iter.peek() {
        Some(& &Delim('/')) => {
            iter.next();
            line_height = match iter.next() {
                Some(v) => line_height::from_component_value(v),
                _ => return None,
            };
            if line_height.is_none() { return None }
        }
        _ => ()
    }
    let family = font_family::from_iter(iter);
    if family.is_none() { return None }
    Some(Longhands{
        font_style: style,
        font_variant: variant,
        font_weight: weight,
        font_size: size,
        line_height: line_height,
        font_family: family
    })
})


four_side_shorthand!(margin, specified::LengthOrPercentageOrAuto::parse,
    margin_top,
    margin_right,
    margin_bottom,
    margin_left
)


four_side_shorthand!(padding, specified::LengthOrPercentage::parse,
    padding_top,
    padding_right,
    padding_bottom,
    padding_left
)
