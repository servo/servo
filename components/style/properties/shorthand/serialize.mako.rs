/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use cssparser::ToCss;
use properties::{AppendableValue, DeclaredValue, PropertyDeclaration, Shorthand};
use values::specified::{BorderStyle, CSSColor};
use std::fmt;

// This macro helps resolve the Optional Property Declaration values into unwrapped values safely
macro_rules! try_unwrap_longhands {
    ( $( $x:ident ),* ) => {
        {
            match (
                $( $x, )*
            ) {

                ( $( Some($x),  )* ) => ( $( $x, )* ),
                _ => return Err(::std::fmt::Error)
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
                   _ => return Err(::std::fmt::Error)
            }
        }
    };
}

fn serialize_four_sides_shorthand<W, I>(dest: &mut W,
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
