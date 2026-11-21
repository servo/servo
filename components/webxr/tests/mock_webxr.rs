/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![cfg(feature = "headless")]

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use embedder_traits::EventLoopWaker;
use euclid::{Point2D, Rect, RigidTransform3D, Size2D, Transform3D, Vector3D};
use profile_traits::generic_callback::GenericCallback as ProfileGenericCallback;
use servo_base::id::PainterId;
use servo_webxr::SurfmanGL;
use servo_webxr::headless::HeadlessMockDiscovery;
use webxr_api::{
    ContextId, Error, Event, GLContexts, Handedness, InputId, InputSource, LayerGrandManager,
    LayerGrandManagerAPI, LayerId, LayerInit, LayerManager, LayerManagerAPI, LayerManagerFactory,
    MainThreadRegistry, MockDeviceInit, MockDeviceMsg, MockInputInit, MockInputMsg, MockViewInit,
    MockViewsInit, SelectEvent, SelectKind, SessionInit, SessionMode, SubImage, SubImages,
    TargetRayMode, Views, Visibility,
};

#[derive(Clone, Default)]
struct TestWaker;

impl EventLoopWaker for TestWaker {
    fn clone_box(&self) -> Box<dyn EventLoopWaker> {
        Box::new(self.clone())
    }

    fn wake(&self) {}
}

#[derive(Clone, Default)]
struct MockLayerGrandManager;

impl LayerGrandManagerAPI<SurfmanGL> for MockLayerGrandManager {
    fn create_layer_manager(
        &self,
        _factory: LayerManagerFactory<SurfmanGL>,
    ) -> Result<LayerManager, Error> {
        Ok(LayerManager::new(MockLayerManager::default()))
    }

    fn clone_layer_grand_manager(&self) -> LayerGrandManager<SurfmanGL> {
        LayerGrandManager::new(self.clone())
    }
}

#[derive(Default)]
struct MockLayerManager {
    layers: Vec<(ContextId, LayerId)>,
}

impl LayerManagerAPI<()> for MockLayerManager {
    fn create_layer(
        &mut self,
        _contexts: &mut dyn GLContexts<()>,
        context_id: ContextId,
        _init: LayerInit,
    ) -> Result<LayerId, Error> {
        let id = LayerId::default();
        self.layers.push((context_id, id));
        Ok(id)
    }

    fn destroy_layer(
        &mut self,
        _contexts: &mut dyn GLContexts<()>,
        context_id: ContextId,
        layer_id: LayerId,
    ) {
        self.layers.retain(|&ids| ids != (context_id, layer_id));
    }

    fn layers(&self) -> &[(ContextId, LayerId)] {
        &self.layers[..]
    }

    fn begin_frame(
        &mut self,
        _contexts: &mut dyn GLContexts<()>,
        layers: &[(ContextId, LayerId)],
    ) -> Result<Vec<SubImages>, Error> {
        Ok(layers
            .iter()
            .map(|&(_, layer_id)| SubImages {
                layer_id,
                sub_image: Some(SubImage {
                    color_texture: None,
                    depth_stencil_texture: None,
                    texture_array_index: None,
                    viewport: Rect::new(Point2D::zero(), Size2D::new(512, 256)),
                }),
                view_sub_images: vec![
                    SubImage {
                        color_texture: None,
                        depth_stencil_texture: None,
                        texture_array_index: None,
                        viewport: Rect::new(Point2D::new(0, 0), Size2D::new(256, 256)),
                    },
                    SubImage {
                        color_texture: None,
                        depth_stencil_texture: None,
                        texture_array_index: None,
                        viewport: Rect::new(Point2D::new(256, 0), Size2D::new(256, 256)),
                    },
                ],
            })
            .collect())
    }

    fn end_frame(
        &mut self,
        _contexts: &mut dyn GLContexts<()>,
        _layers: &[(ContextId, LayerId)],
    ) -> Result<(), Error> {
        Ok(())
    }
}

fn stereo_mock_device_init() -> MockDeviceInit {
    let left_view = MockViewInit {
        transform: RigidTransform3D::from_translation(Vector3D::new(-0.032, 0.0, 0.0)),
        projection: Transform3D::identity(),
        viewport: Rect::new(Point2D::new(0, 0), Size2D::new(256, 256)),
        fov: Some((0.75, 0.75, 0.75, 0.75)),
    };
    let right_view = MockViewInit {
        transform: RigidTransform3D::from_translation(Vector3D::new(0.032, 0.0, 0.0)),
        projection: Transform3D::identity(),
        viewport: Rect::new(Point2D::new(256, 0), Size2D::new(256, 256)),
        fov: Some((0.75, 0.75, 0.75, 0.75)),
    };
    MockDeviceInit {
        floor_origin: Some(RigidTransform3D::from_translation(Vector3D::new(
            0.0, -1.6, 0.0,
        ))),
        supports_inline: false,
        supports_vr: true,
        supports_ar: false,
        viewer_origin: Some(RigidTransform3D::identity()),
        views: MockViewsInit::Stereo(left_view, right_view),
        supported_features: vec!["local-floor".into(), "bounded-floor".into()],
        world: None,
    }
}

fn wait_reply<T: Send + 'static>(
    main_registry: &mut MainThreadRegistry<SurfmanGL>,
    rx: &crossbeam_channel::Receiver<T>,
) -> T {
    let deadline = Instant::now() + Duration::from_secs(5);
    while Instant::now() < deadline {
        main_registry.run_one_frame();
        if let Ok(val) = rx.try_recv() {
            return val;
        }
        std::thread::sleep(Duration::from_millis(2));
    }
    panic!("Timed out waiting for MainThreadRegistry response");
}

#[test]
fn headless_mock_discovery_respects_enabled_flag() {
    let enabled = Arc::new(AtomicBool::new(false));
    let mut main_registry = MainThreadRegistry::new(
        Box::new(TestWaker),
        LayerGrandManager::new(MockLayerGrandManager),
    )
    .unwrap();
    main_registry.register_mock(HeadlessMockDiscovery::new(enabled.clone()));
    let mut registry = main_registry.registry();

    // Disabled mock rejects simulated connections.
    let (tx, rx) = crossbeam_channel::unbounded();
    registry.simulate_device_connection(
        stereo_mock_device_init(),
        ProfileGenericCallback::new(move |res| {
            let _ = tx.send(res.unwrap());
        })
        .unwrap(),
    );
    assert!(matches!(
        wait_reply(&mut main_registry, &rx),
        Err(Error::NoMatchingDevice)
    ));

    // Enabling the mock allows simulated connection.
    enabled.store(true, Ordering::SeqCst);
    let (tx, rx) = crossbeam_channel::unbounded();
    registry.simulate_device_connection(
        stereo_mock_device_init(),
        ProfileGenericCallback::new(move |res| {
            let _ = tx.send(res.unwrap());
        })
        .unwrap(),
    );
    let mock_sender = wait_reply(&mut main_registry, &rx).unwrap();
    let (disc_tx, disc_rx) = crossbeam_channel::unbounded();
    mock_sender
        .send(MockDeviceMsg::Disconnect(
            ProfileGenericCallback::new(move |_| {
                let _ = disc_tx.send(());
            })
            .unwrap(),
        ))
        .unwrap();
    disc_rx.recv_timeout(Duration::from_secs(5)).unwrap();
}

#[test]
fn headless_mock_stereo_session_frames_inputs_and_disconnect() {
    let enabled = Arc::new(AtomicBool::new(true));
    let mut main_registry = MainThreadRegistry::new(
        Box::new(TestWaker),
        LayerGrandManager::new(MockLayerGrandManager),
    )
    .unwrap();
    main_registry.register_mock(HeadlessMockDiscovery::new(enabled));
    let mut registry = main_registry.registry();

    let (tx, rx) = crossbeam_channel::unbounded();
    registry.simulate_device_connection(
        stereo_mock_device_init(),
        ProfileGenericCallback::new(move |res| {
            let _ = tx.send(res.unwrap());
        })
        .unwrap(),
    );
    let mock_sender = wait_reply(&mut main_registry, &rx).unwrap();

    // Verify supports_session for VR vs AR.
    let (tx, rx) = crossbeam_channel::unbounded();
    registry.supports_session(
        SessionMode::ImmersiveVR,
        ProfileGenericCallback::new(move |res| {
            let _ = tx.send(res.unwrap());
        })
        .unwrap(),
    );
    assert!(wait_reply(&mut main_registry, &rx).is_ok());

    let (tx, rx) = crossbeam_channel::unbounded();
    registry.supports_session(
        SessionMode::ImmersiveAR,
        ProfileGenericCallback::new(move |res| {
            let _ = tx.send(res.unwrap());
        })
        .unwrap(),
    );
    assert!(matches!(
        wait_reply(&mut main_registry, &rx),
        Err(Error::NoMatchingDevice)
    ));

    // Request stereo VR session.
    let (tx, rx) = crossbeam_channel::unbounded();
    registry.request_session(
        SessionMode::ImmersiveVR,
        SessionInit {
            required_features: vec!["local-floor".into()],
            optional_features: vec!["bounded-floor".into()],
            first_person_observer_view: false,
            painter_id: PainterId::next(),
        },
        ProfileGenericCallback::new(move |res| {
            let _ = tx.send(res.unwrap());
        })
        .unwrap(),
    );
    let mut session = wait_reply(&mut main_registry, &rx).unwrap();
    assert_eq!(session.viewports().len(), 2);

    let (event_tx, event_rx) = crossbeam_channel::unbounded();
    session.set_event_dest(
        ProfileGenericCallback::new(move |ev| {
            let _ = event_tx.send(ev.unwrap());
        })
        .unwrap(),
    );
    let (frame_tx, frame_rx) = crossbeam_channel::unbounded();
    session.set_frame_dest(
        ProfileGenericCallback::new(move |frame| {
            let _ = frame_tx.send(frame.unwrap());
        })
        .unwrap(),
    );

    let context_id = ContextId(7);
    let layer_id = session
        .create_layer(
            context_id,
            LayerInit::WebGLLayer {
                antialias: true,
                depth: true,
                stencil: false,
                alpha: true,
                ignore_depth_values: false,
                framebuffer_scale_factor: 1.0,
            },
        )
        .unwrap();
    session.set_layers(vec![(context_id, layer_id)]);
    session.start_render_loop();

    let frame = frame_rx.recv_timeout(Duration::from_secs(5)).unwrap();
    assert!(matches!(frame.pose.unwrap().views, Views::Stereo(_, _)));
    assert_eq!(frame.sub_images.len(), 1);
    assert_eq!(frame.sub_images[0].view_sub_images.len(), 2);

    // Send visibility change, input source addition, select trigger, and disconnect.
    mock_sender
        .send(MockDeviceMsg::VisibilityChange(Visibility::VisibleBlurred))
        .unwrap();
    mock_sender
        .send(MockDeviceMsg::AddInputSource(MockInputInit {
            source: InputSource {
                handedness: Handedness::Right,
                id: InputId(0),
                target_ray_mode: TargetRayMode::TrackedPointer,
                supports_grip: true,
                profiles: vec!["generic-trigger".into()],
                hand_support: None,
            },
            pointer_origin: Some(RigidTransform3D::identity()),
            grip_origin: Some(RigidTransform3D::identity()),
            supported_buttons: vec![],
        }))
        .unwrap();
    mock_sender
        .send(MockDeviceMsg::MessageInputSource(
            InputId(0),
            MockInputMsg::TriggerSelect(SelectKind::Select, SelectEvent::Select),
        ))
        .unwrap();

    // Pump frames so the mock device processes the messages and emits events.
    let mut saw_visibility = false;
    let mut saw_add_input = false;
    let mut saw_select = false;
    let deadline = Instant::now() + Duration::from_secs(5);
    while Instant::now() < deadline && !(saw_visibility && saw_add_input && saw_select) {
        session.render_animation_frame();
        let _ = frame_rx.recv_timeout(Duration::from_millis(50));
        while let Ok(ev) = event_rx.try_recv() {
            match ev {
                Event::VisibilityChange(Visibility::VisibleBlurred) => saw_visibility = true,
                Event::AddInput(src) if src.id == InputId(0) => saw_add_input = true,
                Event::Select(InputId(0), SelectKind::Select, SelectEvent::Select, _) => {
                    saw_select = true;
                },
                _ => {},
            }
        }
    }
    assert!(saw_visibility, "expected VisibilityChange event");
    assert!(saw_add_input, "expected AddInput event");
    assert!(saw_select, "expected Select event");

    // Disconnect device -> session should receive Event::SessionEnd.
    let (disc_tx, disc_rx) = crossbeam_channel::unbounded();
    mock_sender
        .send(MockDeviceMsg::Disconnect(
            ProfileGenericCallback::new(move |_| {
                let _ = disc_tx.send(());
            })
            .unwrap(),
        ))
        .unwrap();
    disc_rx.recv_timeout(Duration::from_secs(5)).unwrap();

    let mut saw_session_end = false;
    let deadline = Instant::now() + Duration::from_secs(5);
    while Instant::now() < deadline && !saw_session_end {
        session.render_animation_frame();
        if let Ok(Event::SessionEnd) = event_rx.recv_timeout(Duration::from_millis(50)) {
            saw_session_end = true;
        }
    }
    assert!(saw_session_end, "expected Event::SessionEnd on disconnect");
}
