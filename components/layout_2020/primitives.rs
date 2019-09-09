/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// use crate::text;

// pub use euclid::point2 as point;
// pub use euclid::rect;
// pub type Length<U> = euclid::Length<f32, U>;
pub type Point<U> = euclid::Point2D<f32, U>;
pub type Size<U> = euclid::Size2D<f32, U>;
pub type Rect<U> = euclid::Rect<f32, U>;
// pub type SideOffsets<U> = euclid::SideOffsets2D<f32, U>;
// pub type Scale<Src, Dest> = euclid::Scale<f32, Src, Dest>;

// #[derive(Copy, Clone, PartialEq)]
// pub struct RGBA(pub f32, pub f32, pub f32, pub f32);

// // pub struct TextRun {
// //     pub segment: text::ShapedSegment,
// //     pub font_size: Length<CssPx>,
// //     pub origin: Point<CssPx>,
// // }

// impl From<cssparser::RGBA> for RGBA {
//     fn from(c: cssparser::RGBA) -> Self {
//         RGBA(c.red_f32(), c.green_f32(), c.blue_f32(), c.alpha_f32())
//     }
// }
