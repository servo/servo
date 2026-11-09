use std::thread;
use std::time::Duration;

use euclid::RigidTransform3D;
use profile_traits::generic_callback::GenericCallback;
use webxr_api::{
    ContextId, DeviceAPI, DiscoveryAPI, Error, Event, EventBuffer, Floor, Frame, InputSource,
    LayerId, LayerInit, Native, Quitter, Session, SessionBuilder, SessionInit, SessionMode,
    ViewerPose, Viewports, Views,
};

pub struct InlineDiscovery;

impl<GL: 'static> DiscoveryAPI<GL> for InlineDiscovery {
    fn request_session(
        &mut self,
        mode: SessionMode,
        init: &SessionInit,
        xr: SessionBuilder<GL>,
    ) -> Result<Session, Error> {
        if mode != SessionMode::Inline {
            return Err(Error::NoMatchingDevice);
        }

        let granted_features = init.validate(mode, &[])?;
        xr.spawn(move |_grand_manager| Ok(InlineDevice::new(granted_features)))
    }

    fn supports_session(&self, mode: SessionMode) -> bool {
        mode == SessionMode::Inline
    }
}

pub struct InlineDevice {
    events: EventBuffer,
    granted_features: Vec<String>,
}

impl InlineDevice {
    pub fn new(granted_features: Vec<String>) -> Self {
        Self {
            granted_features,
            events: EventBuffer::default(),
        }
    }
}

impl DeviceAPI for InlineDevice {
    fn create_layer(&mut self, _context_id: ContextId, _init: LayerInit) -> Result<LayerId, Error> {
        Err(Error::InlineSession)
    }

    fn destroy_layer(&mut self, _context_id: ContextId, _layer_id: LayerId) {}

    fn floor_transform(&self) -> Option<RigidTransform3D<f32, Native, Floor>> {
        None
    }

    fn viewports(&self) -> Viewports {
        Viewports { viewports: vec![] }
    }

    fn begin_animation_frame(&mut self, _layers: &[(ContextId, LayerId)]) -> Option<Frame> {
        Some(Frame {
            pose: Some(ViewerPose {
                transform: RigidTransform3D::identity(),
                views: Views::Inline,
            }),
            inputs: vec![],
            events: vec![],
            sub_images: vec![],
            hit_test_results: vec![],
            predicted_display_time: 0.0,
        })
    }

    fn end_animation_frame(&mut self, _layers: &[(ContextId, LayerId)]) {
        thread::sleep(Duration::from_millis(15)); // ~60 FPS
    }

    fn initial_inputs(&self) -> Vec<InputSource> {
        vec![]
    }

    fn set_event_dest(&mut self, dest: GenericCallback<Event>) {
        self.events.upgrade(dest);
    }

    fn quit(&mut self) {
        self.events.callback(Event::SessionEnd);
    }

    fn set_quitter(&mut self, _quitter: Quitter) {}

    fn update_clip_planes(&mut self, _near: f32, _far: f32) {}

    fn granted_features(&self) -> &[String] {
        &self.granted_features
    }
}
