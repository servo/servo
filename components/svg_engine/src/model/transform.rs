/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

pub enum TransformOp {
    Translate(f32, f32),
    Scale(f32, f32),
    Rotate(f32, f32, f32),
    SkewX(f32),
    SkewY(f32),
    Matrix([f32; 6]),
}
