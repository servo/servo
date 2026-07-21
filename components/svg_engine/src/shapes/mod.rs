/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

pub(crate) mod circle;
pub(crate) mod ellipse;
pub(crate) mod line;
pub(crate) mod path;
pub(crate) mod polygon;
pub(crate) mod polyline;
pub(crate) mod rectangle;

use malloc_size_of_derive::MallocSizeOf;

pub use self::circle::Circle;
pub use self::ellipse::Ellipse;
pub use self::line::Line;
pub use self::path::Path;
pub use self::polygon::Polygon;
pub use self::polyline::Polyline;
pub use self::rectangle::Rectangle;

#[derive(Debug, MallocSizeOf)]
pub enum Shape {
    Rect(Rectangle),
    Circle(Circle),
    Ellipse(Ellipse),
    Line(Line),
    Polyline(Polyline),
    Polygon(Polygon),
    Path(Path),
}
