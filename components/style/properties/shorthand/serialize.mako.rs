/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use cssparser::ToCss;
use properties::{AppendableValue, DeclaredValue, PropertyDeclaration, Shorthand};
use values::specified::{BorderStyle, CSSColor};
use std::fmt;

pub fn append_shorthand_value<'a, W, I>(dest: &mut W,
                                        shorthand: Shorthand,
                                        declarations: I)
                                        -> fmt::Result
                                        where W: fmt::Write, I: Iterator<Item=&'a PropertyDeclaration> {
    match shorthand {
        % for shorthand in data.shorthands:
            % if shorthand.camel_case != 'Overflow':
                Shorthand::${shorthand.camel_case} =>
                    try!(serialize_${shorthand.ident}_shorthand(dest, declarations)),
            % endif
        % endfor
        Shorthand::Overflow => { } // overflow is a special case and is not covered here
    };

    Ok(())
}

// This macro helps resolve the Optional Property Declaration values into unwrapped values safely
macro_rules! try_unwrap_longhands {
    ( $( $x:ident ),* ) => {
        {
            match (
                $( $x, )*
            ) {

                ( $( Some($x),  )* ) => ( $( $x, )* ),
                _ => return Err(fmt::Error)
            }
        }
    };
}

// This macro flattens DeclaredValues into their contained values in order to
// perform processing
macro_rules! try_unwrap_declared_values {
    ( $( $x:ident ),* ) => {
        {
            match (
                $( $x, )*
            ) {

                ( $( &DeclaredValue::Value(ref $x),  )* ) => ( $( $x, )* ),
                   _ => return Err(fmt::Error)
            }
        }
    };
}

fn serialize_margin_shorthand<'a, W, I>(dest: &mut W,
                                        declarations: I)
                                        -> fmt::Result
                                        where W: fmt::Write, I: Iterator<Item=&'a PropertyDeclaration> {
    let mut top = None;
    let mut right = None;
    let mut bottom = None;
    let mut left = None;

    for decl in declarations {
        match *decl {
            PropertyDeclaration::MarginTop(ref value) => { top = Some(value); },
            PropertyDeclaration::MarginRight(ref value) => { right = Some(value); },
            PropertyDeclaration::MarginBottom(ref value) => { bottom = Some(value); },
            PropertyDeclaration::MarginLeft(ref value) => { left = Some(value); },
            _ => return Err(fmt::Error)
        }
    }

    let (top, right, bottom, left) = try_unwrap_longhands!(top, right, bottom, left);
    let (top, right, bottom, left) = try_unwrap_declared_values!(top, right, bottom, left);

    serialize_positional_shorthand(dest, top, right, bottom, left)
}

fn serialize_padding_shorthand<'a, W, I>(dest: &mut W,
                                         declarations: I)
                                         -> fmt::Result
                                         where W: fmt::Write, I: Iterator<Item=&'a PropertyDeclaration> {
    let mut top = None;
    let mut right = None;
    let mut bottom = None;
    let mut left = None;

    for decl in declarations {
        match *decl {
            PropertyDeclaration::PaddingTop(ref value) => { top = Some(value); },
            PropertyDeclaration::PaddingRight(ref value) => { right = Some(value); },
            PropertyDeclaration::PaddingBottom(ref value) => { bottom = Some(value); },
            PropertyDeclaration::PaddingLeft(ref value) => { left = Some(value); },
            _ => return Err(fmt::Error)
        }
    }

    let (top, right, bottom, left) = try_unwrap_longhands!(top, right, bottom, left);
    let (top, right, bottom, left) = try_unwrap_declared_values!(top, right, bottom, left);

    serialize_positional_shorthand(dest, top, right, bottom, left)
}

pub fn serialize_overflow_shorthand<'a, W, I>(dest: &mut W,
                                              appendable_value: AppendableValue<'a, I>,
                                              is_first_serialization: &mut bool)
                                              -> fmt::Result
                            where W: fmt::Write, I: Iterator<Item=&'a PropertyDeclaration> {

    let declarations = match appendable_value {
        AppendableValue::DeclarationsForShorthand(_, declarations) => declarations,
        _ => return Err(fmt::Error)
    };

    let mut x_value = None;
    let mut y_value = None;

    // Unfortunately, we must do a sub match on the OverflowX and OverflowY values,
    // because their declared values are not of the same type and therefore
    // we cannot use PartialEq without first extracting the value from the y container

    for decl in declarations {
        match *decl {
            PropertyDeclaration::OverflowX(ref declared_value) => {
                match *declared_value {
                    DeclaredValue::Value(value) => { x_value = Some(value); },
                    _ => return Err(fmt::Error)
                }
            },
            PropertyDeclaration::OverflowY(ref declared_value) => {
                match *declared_value {
                    DeclaredValue::Value(container_value) => { y_value = Some(container_value.0); },
                    _ => return Err(fmt::Error)
                }
            },
            _ => return Err(fmt::Error)
        }
    }


    let (x_value, y_value) = try_unwrap_longhands!(x_value, y_value);

    if x_value == y_value {
        try!(super::append_property_name(dest, "overflow", is_first_serialization));
        try!(write!(dest, ": "));
        try!(x_value.to_css(dest));
    } else {
        try!(super::append_property_name(dest, "overflow-x", is_first_serialization));
        try!(write!(dest, ": "));
        try!(x_value.to_css(dest));
        try!(write!(dest, "; "));

        try!(write!(dest, "overflow-y: "));
        try!(y_value.to_css(dest));
    }

    write!(dest, ";")
}

fn serialize_list_style_shorthand<'a, W, I>(dest: &mut W,
                                            declarations: I)
                                            -> fmt::Result
                                            where W: fmt::Write, I: Iterator<Item=&'a PropertyDeclaration> {

    let mut position = None;
    let mut image = None;
    let mut symbol_type = None;

    for decl in declarations {
        match *decl {
            PropertyDeclaration::ListStylePosition(ref value) => { position = Some(value); },
            PropertyDeclaration::ListStyleImage(ref value) => { image = Some(value); },
            PropertyDeclaration::ListStyleType(ref value) => { symbol_type = Some(value); },
            _ => return Err(fmt::Error)
        }
    }

    let (position, image, symbol_type) = try_unwrap_longhands!(position, image, symbol_type);

    match *position {
        DeclaredValue::Initial => try!(write!(dest, "outside")),
        _ => try!(position.to_css(dest))
    }

    try!(write!(dest, " "));

    match *image {
        DeclaredValue::Initial => try!(write!(dest, "none")),
        _ => try!(image.to_css(dest))
    };

    try!(write!(dest, " "));

    match *symbol_type {
        DeclaredValue::Initial => try!(write!(dest, "disc")),
        _ => try!(symbol_type.to_css(dest))
    };

    Ok(())
}

fn serialize_outline_shorthand<'a, W, I>(dest: &mut W,
                                         declarations: I)
                                         -> fmt::Result
                                         where W: fmt::Write, I: Iterator<Item=&'a PropertyDeclaration> {
    let mut width = None;
    let mut style = None;
    let mut color = None;

    for decl in declarations {
        match *decl {
            PropertyDeclaration::OutlineWidth(ref value) => { width = Some(value); },
            PropertyDeclaration::OutlineStyle(ref value) => { style = Some(value); },
            PropertyDeclaration::OutlineColor(ref value) => { color = Some(value); },
            _ => return Err(fmt::Error)
        }
    }

    let (width, style, color) = try_unwrap_longhands!(width, style, color);

    try!(width.to_css(dest));
    try!(write!(dest, " "));

    match *style {
        DeclaredValue::Initial => try!(write!(dest, "none")),
        _ => try!(style.to_css(dest))
    };

    match *color {
        DeclaredValue::Initial => {},
        _ => {
            try!(write!(dest, " "));
            try!(color.to_css(dest));
        }
    };

    Ok(())
}

// Shorthand::WordWrap name outdated -> https://developer.mozilla.org/en-US/docs/Web/CSS/overflow-wrap
fn serialize_word_wrap_shorthand<'a, W, I>(dest: &mut W,
                                           declarations: I)
                                           -> fmt::Result
                                           where W: fmt::Write, I: Iterator<Item=&'a PropertyDeclaration> {
    let mut overflow_wrap = None;

    for decl in declarations {
        match *decl {
            PropertyDeclaration::OverflowWrap(ref value) => { overflow_wrap = Some(value); },
            _ => return Err(fmt::Error)
        }
    }

    match overflow_wrap {
        Some(overflow_wrap) => overflow_wrap.to_css(dest),
        None => return Err(fmt::Error)
    }
}

fn serialize_flex_flow_shorthand<'a, W, I>(dest: &mut W,
                                           declarations: I)
                                           -> fmt::Result
                                           where W: fmt::Write, I: Iterator<Item=&'a PropertyDeclaration> {
    let mut flex_direction = None;
    let mut flex_wrap = None;

    for decl in declarations {
        match *decl {
            PropertyDeclaration::FlexDirection(ref value) => { flex_direction = Some(value); },
            PropertyDeclaration::FlexWrap(ref value) => { flex_wrap = Some(value); },
            _ => return Err(fmt::Error)
        }
    }

    let (flex_direction, flex_wrap) = try_unwrap_longhands!(flex_direction, flex_wrap);

    match *flex_direction {
        DeclaredValue::Initial => try!(write!(dest, "row")),
        _ => try!(flex_direction.to_css(dest))
    };

    try!(write!(dest, " "));

    match *flex_wrap {
        DeclaredValue::Initial => write!(dest, "nowrap"),
        _ => flex_direction.to_css(dest)
    }
}

fn serialize_flex_shorthand<'a, W, I>(dest: &mut W,
                                      declarations: I) -> fmt::Result
                                      where W: fmt::Write, I: Iterator<Item=&'a PropertyDeclaration> {
    let mut flex_grow = None;
    let mut flex_shrink = None;
    let mut flex_basis = None;

    for decl in declarations {
        match *decl {
            PropertyDeclaration::FlexGrow(ref value) => { flex_grow = Some(value); },
            PropertyDeclaration::FlexShrink(ref value) => { flex_shrink = Some(value); },
            PropertyDeclaration::FlexBasis(ref value) => { flex_basis = Some(value); },
            _ => return Err(fmt::Error)
        }
    }

    let (flex_grow, flex_shrink, flex_basis) = try_unwrap_longhands!(flex_grow, flex_shrink, flex_basis);


    try!(flex_grow.to_css(dest));
    try!(write!(dest, " "));

    try!(flex_shrink.to_css(dest));
    try!(write!(dest, " "));

    flex_basis.to_css(dest)
}

fn serialize_columns_shorthand<'a, W, I>(dest: &mut W,
                                         declarations: I) -> fmt::Result
                                         where W: fmt::Write, I: Iterator<Item=&'a PropertyDeclaration> {
    let mut column_width = None;
    let mut column_count = None;

    for decl in declarations {
        match *decl {
            PropertyDeclaration::ColumnWidth(ref value) => { column_width = Some(value); },
            PropertyDeclaration::ColumnCount(ref value) => { column_count = Some(value); },
            _ => return Err(fmt::Error)
        }
    }

    let (column_width, column_count) = try_unwrap_longhands!(column_width, column_count);

    try!(column_width.to_css(dest));
    try!(write!(dest, " "));

    column_count.to_css(dest)
}

fn serialize_animation_shorthand<'a, W, I>(dest: &mut W,
                                           declarations: I) -> fmt::Result
                                           where W: fmt::Write, I: Iterator<Item=&'a PropertyDeclaration> {

    let mut animation_name = None;
    let mut animation_duration = None;
    let mut animation_timing_function = None;
    let mut animation_iteration_count = None;
    let mut animation_direction = None;
    let mut animation_play_state = None;
    let mut animation_fill_mode = None;
    let mut animation_delay = None;

    for decl in declarations {
        match *decl {
            PropertyDeclaration::AnimationName(ref value) => { animation_name = Some(value); },
            PropertyDeclaration::AnimationDuration(ref value) => { animation_duration = Some(value); },
            PropertyDeclaration::AnimationTimingFunction(ref value) => { animation_timing_function = Some(value); },
            PropertyDeclaration::AnimationIterationCount(ref value) => { animation_iteration_count = Some(value); },
            PropertyDeclaration::AnimationDirection(ref value) => { animation_direction = Some(value); },
            PropertyDeclaration::AnimationPlayState(ref value) => { animation_play_state = Some(value); },
            PropertyDeclaration::AnimationFillMode(ref value) => { animation_fill_mode = Some(value); },
            PropertyDeclaration::AnimationDelay(ref value) => { animation_delay  = Some(value); },
            _ => return Err(fmt::Error)
        }
    }

    let (animation_name, animation_duration, animation_timing_function, animation_iteration_count,
         animation_direction, animation_play_state, animation_fill_mode, animation_delay) =

        try_unwrap_longhands!(animation_name, animation_duration, animation_timing_function,
            animation_iteration_count, animation_direction, animation_play_state,
            animation_fill_mode, animation_delay);

    try!(animation_duration.to_css(dest));
    try!(write!(dest, " "));

    // FIXME: timing function is displaying the actual mathematical name "cubic-bezier(0.25, 0.1, 0.25, 1)" instead
    // of the common name "ease"
    try!(animation_timing_function.to_css(dest));
    try!(write!(dest, " "));

    try!(animation_delay.to_css(dest));
    try!(write!(dest, " "));

    try!(animation_direction.to_css(dest));
    try!(write!(dest, " "));

    try!(animation_fill_mode.to_css(dest));
    try!(write!(dest, " "));

    try!(animation_iteration_count.to_css(dest));
    try!(write!(dest, " "));

    try!(animation_play_state.to_css(dest));
    try!(write!(dest, " "));

    animation_name.to_css(dest)
}

fn serialize_transition_shorthand<'a, W, I>(dest: &mut W,
                                            declarations: I) -> fmt::Result
                                            where W: fmt::Write, I: Iterator<Item=&'a PropertyDeclaration> {
    let mut property_name = None;
    let mut duration = None;
    let mut timing_function = None;
    let mut delay = None;

    for decl in declarations {
        match *decl {
            PropertyDeclaration::TransitionProperty(ref value) => { property_name = Some(value); },
            PropertyDeclaration::TransitionDuration(ref value) => { duration = Some(value); },
            PropertyDeclaration::TransitionTimingFunction(ref value) => { timing_function = Some(value); },
            PropertyDeclaration::TransitionDelay(ref value) => { delay = Some(value); },
            _ => return Err(fmt::Error)
        }
    }

    let (property_name, duration, timing_function, delay) =
        try_unwrap_longhands!(property_name, duration, timing_function, delay);

    try!(property_name.to_css(dest));
    try!(write!(dest, " "));

    try!(duration.to_css(dest));
    try!(write!(dest, " "));

    try!(timing_function.to_css(dest));
    try!(write!(dest, " "));

    delay.to_css(dest)
}

fn serialize_border_width_shorthand<'a, W, I>(dest: &mut W,
                                              declarations: I) -> fmt::Result
                                              where W: fmt::Write, I: Iterator<Item=&'a PropertyDeclaration> {
    let mut top = None;
    let mut right = None;
    let mut bottom = None;
    let mut left = None;

    for decl in declarations {
        match *decl {
            PropertyDeclaration::BorderTopWidth(ref value) => { top = Some(value); },
            PropertyDeclaration::BorderRightWidth(ref value) => { right = Some(value); },
            PropertyDeclaration::BorderBottomWidth(ref value) => { bottom = Some(value); },
            PropertyDeclaration::BorderLeftWidth(ref value) => { left = Some(value); },
            _ => return Err(fmt::Error)
        }
    }

    let (top, right, bottom, left) = try_unwrap_longhands!(top, right, bottom, left);
    let (top, right, bottom, left) = try_unwrap_declared_values!(top, right, bottom, left);

    serialize_positional_shorthand(dest, &top.0, &right.0, &bottom.0, &left.0)
}

// This may be a bit off, possibly needs changes
fn serialize_font_shorthand<'a, W, I>(dest: &mut W,
                                      declarations: I) -> fmt::Result
                                      where W: fmt::Write, I: Iterator<Item=&'a PropertyDeclaration> {

    let mut font_family = None;
    let mut font_style = None;
    let mut font_variant = None;
    let mut font_weight = None;
    let mut font_size = None;
    let mut font_stretch = None;
    let mut line_height = None;

    for decl in declarations {
        match *decl {
            PropertyDeclaration::FontFamily(ref value) => { font_family = Some(value); },
            PropertyDeclaration::FontStyle(ref value) => { font_style = Some(value); },
            PropertyDeclaration::FontVariant(ref value) => { font_variant = Some(value); },
            PropertyDeclaration::FontWeight(ref value) => { font_weight = Some(value); },
            PropertyDeclaration::FontSize(ref value) => { font_size = Some(value); },
            PropertyDeclaration::FontStretch(ref value) => { font_stretch = Some(value); },
            PropertyDeclaration::LineHeight(ref value) => { line_height = Some(value); },
            _ => return Err(fmt::Error)
        }
    }

    let (font_family, font_style, font_variant, font_weight, font_size, font_stretch, line_height) =
        try_unwrap_longhands!(
            font_family, font_style, font_variant, font_weight, font_size, font_stretch, line_height
        );

    if let DeclaredValue::Value(ref style) = *font_style {
        try!(style.to_css(dest));
        try!(write!(dest, " "));
    }

    if let DeclaredValue::Value(ref variant) = *font_variant {
        try!(variant.to_css(dest));
        try!(write!(dest, " "));
    }

    if let DeclaredValue::Value(ref weight) = *font_weight {
        try!(weight.to_css(dest));
        try!(write!(dest, " "));
    }

    if let DeclaredValue::Value(ref stretch) = *font_stretch {
        try!(stretch.to_css(dest));
        try!(write!(dest, " "));
    }

    try!(font_size.to_css(dest));
    if let DeclaredValue::Value(ref height) = *line_height {
        try!(write!(dest, "/"));
        try!(height.to_css(dest));
    }

    try!(write!(dest, " "));

    font_family.to_css(dest)
}

fn serialize_border_top_shorthand<'a, W, I>(dest: &mut W,
                                            declarations: I) -> fmt::Result
                                            where W: fmt::Write, I: Iterator<Item=&'a PropertyDeclaration> {

    let mut width = None;
    let mut style = None;
    let mut color = None;

    for decl in declarations {
        match *decl {
            PropertyDeclaration::BorderTopWidth(ref value) => { width = Some(value); },
            PropertyDeclaration::BorderTopStyle(ref value) => { style = Some(value); },
            PropertyDeclaration::BorderTopColor(ref value) => { color = Some(value); },
            _ => return Err(fmt::Error)
        }
    }

    let (width, style, color) = try_unwrap_longhands!(width, style, color);

    serialize_directional_border_shorthand(dest, width, style, color)
}

fn serialize_border_right_shorthand<'a, W, I>(dest: &mut W,
                                           declarations: I) -> fmt::Result
                                           where W: fmt::Write, I: Iterator<Item=&'a PropertyDeclaration> {

    let mut width = None;
    let mut style = None;
    let mut color = None;

    for decl in declarations {
        match *decl {
            PropertyDeclaration::BorderRightWidth(ref value) => { width = Some(value); },
            PropertyDeclaration::BorderRightStyle(ref value) => { style = Some(value); },
            PropertyDeclaration::BorderRightColor(ref value) => { color = Some(value); },
            _ => return Err(fmt::Error)
        }
    }

    let (width, style, color) = try_unwrap_longhands!(width, style, color);

    serialize_directional_border_shorthand(dest, width, style, color)
}

fn serialize_border_bottom_shorthand<'a, W, I>(dest: &mut W,
                                               declarations: I) -> fmt::Result
                                               where W: fmt::Write, I: Iterator<Item=&'a PropertyDeclaration> {

    let mut width = None;
    let mut style = None;
    let mut color = None;

    for decl in declarations {
        match *decl {
            PropertyDeclaration::BorderBottomWidth(ref value) => { width = Some(value); },
            PropertyDeclaration::BorderBottomStyle(ref value) => { style = Some(value); },
            PropertyDeclaration::BorderBottomColor(ref value) => { color = Some(value); },
            _ => return Err(fmt::Error)
        }
    }

    let (width, style, color) = try_unwrap_longhands!(width, style, color);

    serialize_directional_border_shorthand(dest, width, style, color)
}

fn serialize_border_left_shorthand<'a, W, I>(dest: &mut W,
                                             declarations: I) -> fmt::Result
                                             where W: fmt::Write, I: Iterator<Item=&'a PropertyDeclaration> {

    let mut width = None;
    let mut style = None;
    let mut color = None;

    for decl in declarations {
        match *decl {
            PropertyDeclaration::BorderLeftWidth(ref value) => { width = Some(value); },
            PropertyDeclaration::BorderLeftStyle(ref value) => { style = Some(value); },
            PropertyDeclaration::BorderLeftColor(ref value) => { color = Some(value); },
            _ => return Err(fmt::Error)
        }
    }

    let (width, style, color) = try_unwrap_longhands!(width, style, color);

    serialize_directional_border_shorthand(dest, width, style, color)
}

fn serialize_border_color_shorthand<'a, W, I>(dest: &mut W,
                                              declarations: I) -> fmt::Result
                                              where W: fmt::Write, I: Iterator<Item=&'a PropertyDeclaration> {

    let mut top = None;
    let mut right = None;
    let mut bottom = None;
    let mut left = None;

    for decl in declarations {
        match *decl {
            PropertyDeclaration::BorderTopColor(ref value) => { top = Some(value); },
            PropertyDeclaration::BorderRightColor(ref value) => { right = Some(value); },
            PropertyDeclaration::BorderBottomColor(ref value) => { bottom = Some(value); },
            PropertyDeclaration::BorderLeftColor(ref value) => { left = Some(value); },
            _ => return Err(fmt::Error)
        }
    }

    let (top, right, bottom, left) = try_unwrap_longhands!(top, right, bottom, left);

    serialize_positional_shorthand(dest, top, right, bottom, left)
}

fn serialize_border_style_shorthand<'a, W, I>(dest: &mut W,
                                              declarations: I) -> fmt::Result
                                              where W: fmt::Write, I: Iterator<Item=&'a PropertyDeclaration> {

    let mut top = None;
    let mut right = None;
    let mut bottom = None;
    let mut left = None;

    for decl in declarations {
        match *decl {
            PropertyDeclaration::BorderTopStyle(ref value) => { top = Some(value); },
            PropertyDeclaration::BorderRightStyle(ref value) => { right = Some(value); },
            PropertyDeclaration::BorderBottomStyle(ref value) => { bottom = Some(value); },
            PropertyDeclaration::BorderLeftStyle(ref value) => { left = Some(value); },
            _ => return Err(fmt::Error)
        }
    }

    let (top, right, bottom, left) = try_unwrap_longhands!(top, right, bottom, left);

    serialize_positional_shorthand(dest, top, right, bottom, left)
}

// TODO: I do not understand how border radius works with respect to the slashes /, so skipping it for now
// https://developer.mozilla.org/en-US/docs/Web/CSS/border-radius
fn serialize_border_radius_shorthand<'a, W, I>(dest: &mut W,
                                               declarations: I) -> fmt::Result
                                               where W: fmt::Write, I: Iterator<Item=&'a PropertyDeclaration> {

    let mut top_left = None;
    let mut top_right = None;
    let mut bottom_right = None;
    let mut bottom_left = None;

    for decl in declarations {
        match *decl {
            PropertyDeclaration::BorderTopLeftRadius(ref value) => { top_left = Some(value); },
            PropertyDeclaration::BorderTopRightRadius(ref value) => { top_right = Some(value); },
            PropertyDeclaration::BorderBottomRightRadius(ref value) => { bottom_right  = Some(value); },
            PropertyDeclaration::BorderBottomLeftRadius(ref value) => { bottom_left = Some(value); },
            _ => return Err(fmt::Error)
        }
    }

    let (top_left, top_right, bottom_right, bottom_left) =
        try_unwrap_longhands!(top_left, top_right, bottom_right, bottom_left);


    try!(top_left.to_css(dest));
    try!(write!(dest, " "));

    try!(top_right.to_css(dest));
    try!(write!(dest, " "));

    try!(bottom_right.to_css(dest));
    try!(write!(dest, " "));

    bottom_left.to_css(dest)
}

fn serialize_border_shorthand<'a, W, I>(dest: &mut W,
                                        declarations: I) -> fmt::Result
                                        where W: fmt::Write, I: Iterator<Item=&'a PropertyDeclaration> {
    let mut top_color = None;
    let mut top_style = None;
    let mut top_width = None;

    let mut right_color = None;
    let mut right_style = None;
    let mut right_width = None;

    let mut bottom_color = None;
    let mut bottom_style = None;
    let mut bottom_width = None;

    let mut left_color = None;
    let mut left_style = None;
    let mut left_width = None;

    for decl in declarations {
        match *decl {
            PropertyDeclaration::BorderTopColor(ref value) => { top_color = Some(value); },
            PropertyDeclaration::BorderTopStyle(ref value) => { top_style = Some(value); },
            PropertyDeclaration::BorderTopWidth(ref value) => { top_width  = Some(value); },
            PropertyDeclaration::BorderRightColor(ref value) => { right_color = Some(value); },
            PropertyDeclaration::BorderRightStyle(ref value) => { right_style = Some(value); },
            PropertyDeclaration::BorderRightWidth(ref value) => { right_width  = Some(value); },
            PropertyDeclaration::BorderBottomColor(ref value) => { bottom_color = Some(value); },
            PropertyDeclaration::BorderBottomStyle(ref value) => { bottom_style = Some(value); },
            PropertyDeclaration::BorderBottomWidth(ref value) => { bottom_width  = Some(value); },
            PropertyDeclaration::BorderLeftColor(ref value) => { left_color = Some(value); },
            PropertyDeclaration::BorderLeftStyle(ref value) => { left_style = Some(value); },
            PropertyDeclaration::BorderLeftWidth(ref value) => { left_width  = Some(value); },
            _ => return Err(fmt::Error)
        }
    }

    // Check that the longhands are all present, but once the check is done
    // they should all be the same, we can just one set of color/style/width
    let (
        top_color, top_style, top_width,
        _, _, _,
        _, _, _,
        _, _, _
    ) =
    try_unwrap_longhands!(
        top_color, top_style, top_width,
        right_color, right_style, right_width,
        bottom_color, bottom_style, bottom_width,
        left_color, left_style, left_width
    );

    serialize_directional_border_shorthand(dest, top_width, top_style, top_color)
}

// Implementation may be a bit off, likely needs some changes
fn serialize_background_shorthand<'a, W, I>(dest: &mut W,
                                            declarations: I) -> fmt::Result
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

    if let DeclaredValue::Value(ref origin) = *origin {
        try!(write!(dest, " "));
        try!(origin.to_css(dest));
    }

    if let DeclaredValue::Value(ref clip) = *clip {
        try!(write!(dest, " "));
        try!(clip.to_css(dest));
    }

    Ok(())
}

fn serialize_positional_shorthand<W, I>(dest: &mut W,
                                        top: &I,
                                        right: &I,
                                        bottom: &I,
                                        left: &I)
                                        -> fmt::Result where W: fmt::Write, I: ToCss + PartialEq {

      if left == right {
          let horizontal_value = left;

          if top == bottom {
              let vertical_value = top;

               if horizontal_value == vertical_value {
                   let single_value = horizontal_value;
                   try!(single_value.to_css(dest));
               } else {
                   try!(vertical_value.to_css(dest));
                   try!(write!(dest, " "));

                   try!(horizontal_value.to_css(dest));
               }
          } else {
              try!(top.to_css(dest));
              try!(write!(dest, " "));

              try!(horizontal_value.to_css(dest));
              try!(write!(dest, " "));

              try!(bottom.to_css(dest));
          }
      } else {
          try!(top.to_css(dest));
          try!(write!(dest, " "));

          try!(right.to_css(dest));
          try!(write!(dest, " "));

          try!(bottom.to_css(dest));
          try!(write!(dest, " "));

          try!(left.to_css(dest));
      }

      Ok(())
}

fn serialize_directional_border_shorthand<W, I>(dest: &mut W,
                                                width: &DeclaredValue<I>,
                                                style: &DeclaredValue<BorderStyle>,
                                                color: &DeclaredValue<CSSColor>)
                                                -> fmt::Result where W: fmt::Write, I: ToCss {
    match *width {
        DeclaredValue::Value(ref width) => {
            try!(width.to_css(dest));
        },
        _ => {
            try!(write!(dest, "medium"));
        }
    };

    try!(write!(dest, " "));

    match *style {
        DeclaredValue::Value(ref style) => {
            try!(style.to_css(dest));
        },
        _ => {
            try!(write!(dest, "none"));
        }
    };

    match *color {
        DeclaredValue::Value(ref color) => {
            try!(write!(dest, " "));
            color.to_css(dest)
        },
        _ => Ok(())
    }
}


pub fn is_overflow_shorthand<'a, I>(appendable_value: &AppendableValue<'a, I>) -> bool
                                    where I: Iterator<Item=&'a PropertyDeclaration> {
    if let AppendableValue::DeclarationsForShorthand(shorthand, _) = *appendable_value {
        if let Shorthand::Overflow = shorthand {
            return true;
        }
    }

    false
}
