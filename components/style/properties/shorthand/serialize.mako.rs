/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use style_traits::{CssWriter, ToCss};
use values::specified::{BorderStyle, Color};
use std::fmt::{self, Write};

fn serialize_directional_border<W, I,>(
    dest: &mut CssWriter<W>,
    width: &I,
    style: &BorderStyle,
    color: &Color,
) -> fmt::Result
where
    W: Write,
    I: ToCss,
{
    width.to_css(dest)?;
    dest.write_str(" ")?;
    style.to_css(dest)?;
    if *color != Color::CurrentColor {
        dest.write_str(" ")?;
        color.to_css(dest)?;
    }
    Ok(())
}
