/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// This module implements a test WebVR display, which renders to the screen
// in the same way as fullscreen does. It gets its render size from the current
// size of the window.

use euclid::Angle;
use euclid::Trig;
use euclid::TypedSize2D;
use rust_webvr_api::VRDisplay;
use rust_webvr_api::VRDisplayCapabilities;
use rust_webvr_api::VRDisplayData;
use rust_webvr_api::VRDisplayPtr;
use rust_webvr_api::VREvent;
use rust_webvr_api::VREyeParameters;
use rust_webvr_api::VRFieldOfView;
use rust_webvr_api::VRFrameData;
use rust_webvr_api::VRFramebuffer;
use rust_webvr_api::VRFramebufferAttributes;
use rust_webvr_api::VRGamepadPtr;
use rust_webvr_api::VRLayer;
use rust_webvr_api::VRService;
use rust_webvr_api::VRViewport;
use std::cell::RefCell;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering::Relaxed;
use std::sync::Arc;
use style_traits::DevicePixel;

pub type TestVRDisplayPtr = Arc<RefCell<TestVRDisplay>>;

// This is very very unsafe, but the API requires it.
#[allow(unsafe_code)]
unsafe impl Sync for TestVRService {}
#[allow(unsafe_code)]
unsafe impl Send for TestVRService {}

pub struct TestVRService {
    display: Option<TestVRDisplayPtr>,
    size: TypedSize2D<u32, DevicePixel>,
}

pub struct TestVRDisplay {
    timestamp: AtomicUsize,
    size: TypedSize2D<u32, DevicePixel>,
}

// Each Test device only has one display
const DISPLAY_ID: u32 = 1;
const DISPLAY_NAME: &str = "Test Display";

// Fake a display with a distance between eyes of 5cm.
const EYE_DISTANCE: f32 = 0.05;

impl VRDisplay for TestVRDisplay {
    fn id(&self) -> u32 {
        DISPLAY_ID
    }

    fn data(&self) -> VRDisplayData {
        let capabilities = VRDisplayCapabilities {
            has_position: false,
            has_orientation: false,
            has_external_display: false,
            can_present: true,
            max_layers: 1,
        };

        let fov_right = self.fov_right().to_degrees();
        let fov_up = self.fov_up().to_degrees();

        let field_of_view = VRFieldOfView {
            down_degrees: fov_up,
            left_degrees: fov_right,
            right_degrees: fov_right,
            up_degrees: fov_up,
        };

        let left_eye_parameters = VREyeParameters {
            offset: [-EYE_DISTANCE / 2.0, 0.0, 0.0],
            render_width: self.size.width / 2,
            render_height: self.size.height,
            field_of_view: field_of_view,
        };

        let right_eye_parameters = VREyeParameters {
            offset: [EYE_DISTANCE / 2.0, 0.0, 0.0],
            ..left_eye_parameters.clone()
        };

        VRDisplayData {
            display_id: DISPLAY_ID,
            display_name: String::from(DISPLAY_NAME),
            connected: true,
            capabilities: capabilities,
            stage_parameters: None,
            left_eye_parameters: left_eye_parameters,
            right_eye_parameters: right_eye_parameters,
        }
    }

    fn inmediate_frame_data(&self, near: f64, far: f64) -> VRFrameData {
        let left_projection_matrix = self.perspective(near, far);
        let right_projection_matrix = left_projection_matrix.clone();

        VRFrameData {
            timestamp: self.timestamp.fetch_add(1, Relaxed) as f64,
            // TODO: adjust matrices for stereoscopic vision
            left_projection_matrix: left_projection_matrix,
            right_projection_matrix: right_projection_matrix,
            ..VRFrameData::default()
        }
    }

    fn synced_frame_data(&self, near: f64, far: f64) -> VRFrameData {
        self.inmediate_frame_data(near, far)
    }

    fn reset_pose(&mut self) {}

    fn sync_poses(&mut self) {}

    fn bind_framebuffer(&mut self, _eye_index: u32) {}

    fn get_framebuffers(&self) -> Vec<VRFramebuffer> {
        let left_viewport = VRViewport {
            x: 0,
            y: 0,
            width: (self.size.width as i32) / 2,
            height: self.size.height as i32,
        };

        let right_viewport = VRViewport {
            x: self.size.width as i32 - left_viewport.width,
            ..left_viewport
        };

        vec![
            VRFramebuffer {
                eye_index: 0,
                attributes: VRFramebufferAttributes::default(),
                viewport: left_viewport,
            },
            VRFramebuffer {
                eye_index: 1,
                attributes: VRFramebufferAttributes::default(),
                viewport: right_viewport,
            },
        ]
    }

    fn render_layer(&mut self, layer: &VRLayer) {
        if let Some((width, height)) = layer.texture_size {
            self.size = TypedSize2D::new(width, height);
        }
    }

    fn submit_frame(&mut self) {}

    fn start_present(&mut self, _attributes: Option<VRFramebufferAttributes>) {}

    fn stop_present(&mut self) {}
}

impl TestVRDisplay {
    pub fn new(size: TypedSize2D<u32, DevicePixel>) -> TestVRDisplay {
        TestVRDisplay {
            timestamp: AtomicUsize::new(0),
            size: size,
        }
    }

    fn fov_up(&self) -> Angle<f64> {
        Angle::radians(f64::fast_atan2(
            2.0 * self.size.height as f64,
            self.size.width as f64,
        ))
    }

    fn fov_right(&self) -> Angle<f64> {
        Angle::radians(f64::fast_atan2(
            2.0 * self.size.width as f64,
            self.size.height as f64,
        ))
    }

    fn perspective(&self, near: f64, far: f64) -> [f32; 16] {
        // https://github.com/toji/gl-matrix/blob/bd3307196563fbb331b40fc6ebecbbfcc2a4722c/src/mat4.js#L1271
        let near = near as f32;
        let far = far as f32;
        let f = 1.0 / self.fov_up().radians.tan() as f32;
        let nf = 1.0 / (near - far);
        let aspect = ((self.size.width / 2) as f32) / (self.size.height as f32);

        // Dear rustfmt, This is a 4x4 matrix, please leave it alone. Best, ajeffrey.
        #[rustfmt::skip]
        [
            f / aspect, 0.0, 0.0,                   0.0,
            0.0,        f,   0.0,                   0.0,
            0.0,        0.0, (far + near) * nf,     -1.0,
            0.0,        0.0, 2.0 * far * near * nf, 0.0,
        ]
    }
}

impl VRService for TestVRService {
    fn initialize(&mut self) -> Result<(), String> {
        if self.display.is_some() {
            return Ok(());
        }

        self.display = Some(Arc::new(RefCell::new(TestVRDisplay::new(self.size))));

        Ok(())
    }

    fn fetch_displays(&mut self) -> Result<Vec<VRDisplayPtr>, String> {
        self.initialize()?;
        self.display
            .iter()
            .map(|display| Ok(display.clone() as VRDisplayPtr))
            .collect()
    }

    fn fetch_gamepads(&mut self) -> Result<Vec<VRGamepadPtr>, String> {
        self.initialize()?;
        Ok(vec![])
    }

    fn is_available(&self) -> bool {
        true
    }

    fn poll_events(&self) -> Vec<VREvent> {
        Vec::new()
    }
}

impl TestVRService {
    pub fn new(size: TypedSize2D<u32, DevicePixel>) -> TestVRService {
        TestVRService {
            display: None,
            size: size,
        }
    }
}
