/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Traits to be implemented by backends

use crate::ContextId;
use crate::EnvironmentBlendMode;
use crate::Error;
use crate::Event;
use crate::Floor;
use crate::Frame;
use crate::HitTestId;
use crate::HitTestSource;
use crate::InputSource;
use crate::LayerId;
use crate::LayerInit;
use crate::Native;
use crate::Quitter;
use crate::Sender;
use crate::Session;
use crate::SessionBuilder;
use crate::SessionInit;
use crate::SessionMode;
use crate::Viewports;

use euclid::{Point2D, RigidTransform3D};

/// A trait for discovering XR devices
pub trait DiscoveryAPI<GL>: 'static {
    fn request_session(
        &mut self,
        mode: SessionMode,
        init: &SessionInit,
        xr: SessionBuilder<GL>,
    ) -> Result<Session, Error>;
    fn supports_session(&self, mode: SessionMode) -> bool;
}

/// A trait for using an XR device
pub trait DeviceAPI: 'static {
    /// Create a new layer
    fn create_layer(&mut self, context_id: ContextId, init: LayerInit) -> Result<LayerId, Error>;

    /// Destroy a layer
    fn destroy_layer(&mut self, context_id: ContextId, layer_id: LayerId);

    /// The transform from native coordinates to the floor.
    fn floor_transform(&self) -> Option<RigidTransform3D<f32, Native, Floor>>;

    fn viewports(&self) -> Viewports;

    /// Begin an animation frame.
    fn begin_animation_frame(&mut self, layers: &[(ContextId, LayerId)]) -> Option<Frame>;

    /// End an animation frame, render the layer to the device, and block waiting for the next frame.
    fn end_animation_frame(&mut self, layers: &[(ContextId, LayerId)]);

    /// Inputs registered with the device on initialization. More may be added, which
    /// should be communicated through a yet-undecided event mechanism
    fn initial_inputs(&self) -> Vec<InputSource>;

    /// Sets the event handling channel
    fn set_event_dest(&mut self, dest: Sender<Event>);

    /// Quit the session
    fn quit(&mut self);

    fn set_quitter(&mut self, quitter: Quitter);

    fn update_clip_planes(&mut self, near: f32, far: f32);

    fn environment_blend_mode(&self) -> EnvironmentBlendMode {
        // for VR devices, override for AR
        EnvironmentBlendMode::Opaque
    }

    fn granted_features(&self) -> &[String];

    fn request_hit_test(&mut self, _source: HitTestSource) {
        panic!("This device does not support requesting hit tests");
    }

    fn cancel_hit_test(&mut self, _id: HitTestId) {
        panic!("This device does not support hit tests");
    }

    fn update_frame_rate(&mut self, rate: f32) -> f32 {
        rate
    }

    fn supported_frame_rates(&self) -> Vec<f32> {
        Vec::new()
    }

    fn reference_space_bounds(&self) -> Option<Vec<Point2D<f32, Floor>>> {
        None
    }
}

impl<GL: 'static> DiscoveryAPI<GL> for Box<dyn DiscoveryAPI<GL>> {
    fn request_session(
        &mut self,
        mode: SessionMode,
        init: &SessionInit,
        xr: SessionBuilder<GL>,
    ) -> Result<Session, Error> {
        (&mut **self).request_session(mode, init, xr)
    }

    fn supports_session(&self, mode: SessionMode) -> bool {
        (&**self).supports_session(mode)
    }
}
