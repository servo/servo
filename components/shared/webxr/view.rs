/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! This crate uses `euclid`'s typed units, and exposes different coordinate spaces.

use euclid::Rect;
use euclid::RigidTransform3D;
use euclid::Transform3D;

#[cfg(feature = "ipc")]
use serde::{Deserialize, Serialize};

use std::marker::PhantomData;

/// The coordinate space of the viewer
/// https://immersive-web.github.io/webxr/#dom-xrreferencespacetype-viewer
#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "ipc", derive(Serialize, Deserialize))]
pub enum Viewer {}

/// The coordinate space of the floor
/// https://immersive-web.github.io/webxr/#dom-xrreferencespacetype-local-floor
#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "ipc", derive(Serialize, Deserialize))]
pub enum Floor {}

/// The coordinate space of the left eye
/// https://immersive-web.github.io/webxr/#dom-xreye-left
#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "ipc", derive(Serialize, Deserialize))]
pub enum LeftEye {}

/// The coordinate space of the right eye
/// https://immersive-web.github.io/webxr/#dom-xreye-right
#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "ipc", derive(Serialize, Deserialize))]
pub enum RightEye {}

/// The coordinate space of the left frustrum of a cubemap
#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "ipc", derive(Serialize, Deserialize))]
pub enum CubeLeft {}

/// The coordinate space of the right frustrum of a cubemap
#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "ipc", derive(Serialize, Deserialize))]
pub enum CubeRight {}

/// The coordinate space of the top frustrum of a cubemap
#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "ipc", derive(Serialize, Deserialize))]
pub enum CubeTop {}

/// The coordinate space of the bottom frustrum of a cubemap
#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "ipc", derive(Serialize, Deserialize))]
pub enum CubeBottom {}

/// The coordinate space of the back frustrum of a cubemap
#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "ipc", derive(Serialize, Deserialize))]
pub enum CubeBack {}

/// Pattern-match on eyes
#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "ipc", derive(Serialize, Deserialize))]
pub struct SomeEye<Eye>(u8, PhantomData<Eye>);
pub const LEFT_EYE: SomeEye<LeftEye> = SomeEye(0, PhantomData);
pub const RIGHT_EYE: SomeEye<RightEye> = SomeEye(1, PhantomData);
pub const VIEWER: SomeEye<Viewer> = SomeEye(2, PhantomData);
pub const CUBE_LEFT: SomeEye<CubeLeft> = SomeEye(3, PhantomData);
pub const CUBE_RIGHT: SomeEye<CubeRight> = SomeEye(4, PhantomData);
pub const CUBE_TOP: SomeEye<CubeTop> = SomeEye(5, PhantomData);
pub const CUBE_BOTTOM: SomeEye<CubeBottom> = SomeEye(6, PhantomData);
pub const CUBE_BACK: SomeEye<CubeBack> = SomeEye(7, PhantomData);

impl<Eye1, Eye2> PartialEq<SomeEye<Eye2>> for SomeEye<Eye1> {
    fn eq(&self, rhs: &SomeEye<Eye2>) -> bool {
        self.0 == rhs.0
    }
}

/// The native 3D coordinate space of the device
/// This is not part of the webvr specification.
#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "ipc", derive(Serialize, Deserialize))]
pub enum Native {}

/// The normalized device coordinate space, where the display
/// is from (-1,-1) to (1,1).
// TODO: are we OK assuming that we can use the same coordinate system for all displays?
#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "ipc", derive(Serialize, Deserialize))]
pub enum Display {}

/// The unnormalized device coordinate space, where the display
/// is from (0,0) to (w,h), measured in pixels.
// TODO: are we OK assuming that we can use the same coordinate system for all displays?
#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "ipc", derive(Serialize, Deserialize))]
pub enum Viewport {}

/// The coordinate space of an input device
#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "ipc", derive(Serialize, Deserialize))]
pub enum Input {}

/// The coordinate space of a secondary capture view
#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "ipc", derive(Serialize, Deserialize))]
pub enum Capture {}

/// For each eye, the pose of that eye,
/// its projection onto its display.
/// For stereo displays, we have a `View<LeftEye>` and a `View<RightEye>`.
/// For mono displays, we hagve a `View<Viewer>`
/// https://immersive-web.github.io/webxr/#xrview
#[derive(Clone, Debug)]
#[cfg_attr(feature = "ipc", derive(Serialize, Deserialize))]
pub struct View<Eye> {
    pub transform: RigidTransform3D<f32, Eye, Native>,
    pub projection: Transform3D<f32, Eye, Display>,
}

impl<Eye> Default for View<Eye> {
    fn default() -> Self {
        View {
            transform: RigidTransform3D::identity(),
            projection: Transform3D::identity(),
        }
    }
}

impl<Eye> View<Eye> {
    pub fn cast_unit<NewEye>(&self) -> View<NewEye> {
        View {
            transform: self.transform.cast_unit(),
            projection: Transform3D::from_untyped(&self.projection.to_untyped()),
        }
    }
}

/// Whether a device is mono or stereo, and the views it supports.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "ipc", derive(Serialize, Deserialize))]
pub enum Views {
    /// Mono view for inline VR, viewport and projection matrices are calculated by client
    Inline,
    Mono(View<Viewer>),
    Stereo(View<LeftEye>, View<RightEye>),
    StereoCapture(View<LeftEye>, View<RightEye>, View<Capture>),
    Cubemap(
        View<Viewer>,
        View<CubeLeft>,
        View<CubeRight>,
        View<CubeTop>,
        View<CubeBottom>,
        View<CubeBack>,
    ),
}

/// A list of viewports per-eye in the order of fields in Views.
///
/// Not all must be in active use.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "ipc", derive(Serialize, Deserialize))]
pub struct Viewports {
    pub viewports: Vec<Rect<i32, Viewport>>,
}
