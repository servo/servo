/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

pub mod render_tree;
pub mod shapes;

mod renderer;
mod traversal;

pub use render_tree::SvgTag;
pub use traversal::render_svg_tree;
