/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use properties::{AppendableValue, DeclaredValue, PropertyDeclaration, ShorthandId};
use style_traits::ToCss;
use values::specified::{BorderStyle, CSSColor};
use std::fmt;

#[allow(missing_docs)]
pub fn serialize_four_sides<W, I>(dest: &mut W,
                                  top: &I,
                                  right: &I,
                                  bottom: &I,
                                  left: &I)
                                  -> fmt::Result
    where W: fmt::Write,
          I: ToCss + PartialEq,
{

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

fn serialize_directional_border<W, I>(dest: &mut W,
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


#[allow(missing_docs)]
pub fn is_overflow_shorthand<'a, I>(appendable_value: &AppendableValue<'a, I>) -> bool
    where I: Iterator<Item=&'a PropertyDeclaration>
{
    if let AppendableValue::DeclarationsForShorthand(shorthand, _) = *appendable_value {
        if let ShorthandId::Overflow = shorthand {
            return true;
        }
    }

    false
}
