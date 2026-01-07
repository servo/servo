/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use euclid::RigidTransform3D;
use serde::{Deserialize, Serialize};

use crate::{InputId, Joint};

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
/// A stand-in type for "the space isn't statically known since
/// it comes from client side code"
pub struct ApiSpace;

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum BaseSpace {
    Local,
    Floor,
    Viewer,
    BoundedFloor,
    TargetRay(InputId),
    Grip(InputId),
    Joint(InputId, Joint),
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Space {
    pub base: BaseSpace,
    pub offset: RigidTransform3D<f32, ApiSpace, ApiSpace>,
}
