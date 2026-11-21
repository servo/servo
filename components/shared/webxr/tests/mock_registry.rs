/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, mpsc};
use std::time::{Duration, Instant};

use embedder_traits::EventLoopWaker;
use euclid::{Point2D, Rect, RigidTransform3D, Size2D, Transform3D, Vector3D};
use profile_traits::generic_callback::GenericCallback as ProfileGenericCallback;
use servo_base::generic_channel::GenericCallback;
use servo_base::id::{PainterId, PipelineId, PipelineNamespace, PipelineNamespaceId};
use servo_webxr_api::util::{ClipPlanes, fov_to_projection_matrix};
use servo_webxr_api::{
    ContextId, DeviceAPI, DiscoveryAPI, EnvironmentBlendMode, Error, Event, EventBuffer, Floor,
    Frame, GLContexts, LayerGrandManager, LayerGrandManagerAPI, LayerId, LayerInit, LayerManager,
    LayerManagerAPI, LayerManagerFactory, LeftEye, MainThreadRegistry, Native, Quitter, RightEye,
    Session, SessionBuilder, SessionInit, SessionMode, SubImage, SubImages, View, ViewerPose,
    Viewport, Viewports, Views,
};

#[derive(Clone, Default)]
struct MockWaker {
    wakes: Arc<AtomicUsize>,
}

impl EventLoopWaker for MockWaker {
    fn clone_box(&self) -> Box<dyn EventLoopWaker> {
        Box::new(self.clone())
    }

    fn wake(&self) {
        self.wakes.fetch_add(1, Ordering::SeqCst);
    }
}

#[derive(Clone, Default)]
struct NoopLayerGrandManager;

impl LayerGrandManagerAPI<()> for NoopLayerGrandManager {
    fn create_layer_manager(
        &self,
        factory: LayerManagerFactory<()>,
    ) -> Result<LayerManager, Error> {
        let manager = factory.build(&mut ())?;
        Ok(LayerManager::new(MockLayerManagerAdapter(manager)))
    }

    fn clone_layer_grand_manager(&self) -> LayerGrandManager<()> {
        LayerGrandManager::new(self.clone())
    }
}

struct MockLayerManagerAdapter(Box<dyn LayerManagerAPI<()>>);

// SAFETY: Used only with unit `()` mock layer manager in tests.
unsafe impl Send for MockLayerManagerAdapter {}

impl LayerManagerAPI<()> for MockLayerManagerAdapter {
    fn create_layer(
        &mut self,
        contexts: &mut dyn GLContexts<()>,
        context_id: ContextId,
        init: LayerInit,
    ) -> Result<LayerId, Error> {
        self.0.create_layer(contexts, context_id, init)
    }

    fn destroy_layer(
        &mut self,
        contexts: &mut dyn GLContexts<()>,
        context_id: ContextId,
        layer_id: LayerId,
    ) {
        self.0.destroy_layer(contexts, context_id, layer_id)
    }

    fn layers(&self) -> &[(ContextId, LayerId)] {
        self.0.layers()
    }

    fn begin_frame(
        &mut self,
        contexts: &mut dyn GLContexts<()>,
        layers: &[(ContextId, LayerId)],
    ) -> Result<Vec<SubImages>, Error> {
        self.0.begin_frame(contexts, layers)
    }

    fn end_frame(
        &mut self,
        contexts: &mut dyn GLContexts<()>,
        layers: &[(ContextId, LayerId)],
    ) -> Result<(), Error> {
        self.0.end_frame(contexts, layers)
    }
}

struct MockDevice {
    events: EventBuffer,
    granted_features: Vec<String>,
    layers: Vec<(ContextId, LayerId)>,
    clip_planes: ClipPlanes,
    frame_rate: f32,
    frames_begun: Arc<AtomicUsize>,
    frames_ended: Arc<AtomicUsize>,
    quit_called: Arc<AtomicBool>,
}

impl MockDevice {
    fn new(
        granted_features: Vec<String>,
        frames_begun: Arc<AtomicUsize>,
        frames_ended: Arc<AtomicUsize>,
        quit_called: Arc<AtomicBool>,
    ) -> Self {
        Self {
            events: EventBuffer::default(),
            granted_features,
            layers: Vec::new(),
            clip_planes: ClipPlanes::default(),
            frame_rate: 90.0,
            frames_begun,
            frames_ended,
            quit_called,
        }
    }
}

impl DeviceAPI for MockDevice {
    fn floor_transform(&self) -> Option<RigidTransform3D<f32, Native, Floor>> {
        Some(RigidTransform3D::from_translation(Vector3D::new(
            0.0, 1.6, 0.0,
        )))
    }

    fn viewports(&self) -> Viewports {
        Viewports {
            viewports: vec![
                Rect::new(Point2D::new(0, 0), Size2D::new(512, 512)),
                Rect::new(Point2D::new(512, 0), Size2D::new(512, 512)),
            ],
        }
    }

    fn create_layer(&mut self, context_id: ContextId, _init: LayerInit) -> Result<LayerId, Error> {
        let layer_id = LayerId::default();
        self.layers.push((context_id, layer_id));
        Ok(layer_id)
    }

    fn destroy_layer(&mut self, context_id: ContextId, layer_id: LayerId) {
        self.layers.retain(|&ids| ids != (context_id, layer_id));
    }

    fn begin_animation_frame(&mut self, layers: &[(ContextId, LayerId)]) -> Option<Frame> {
        self.frames_begun.fetch_add(1, Ordering::SeqCst);
        let projection = fov_to_projection_matrix(-0.8, 0.8, 0.8, -0.8, self.clip_planes);
        let left = View::<LeftEye> {
            transform: RigidTransform3D::from_translation(Vector3D::new(-0.032, 0.0, 0.0)),
            projection,
        };
        let right = View::<RightEye> {
            transform: RigidTransform3D::from_translation(Vector3D::new(0.032, 0.0, 0.0)),
            projection: Transform3D::from_array(projection.to_array()),
        };
        let viewports = self.viewports().viewports;
        let sub_images = layers
            .iter()
            .map(|&(_, layer_id)| SubImages {
                layer_id,
                sub_image: Some(SubImage {
                    color_texture: None,
                    depth_stencil_texture: None,
                    texture_array_index: None,
                    viewport: Rect::new(Point2D::zero(), Size2D::new(1024, 512)),
                }),
                view_sub_images: viewports
                    .iter()
                    .map(|&viewport| SubImage {
                        color_texture: None,
                        depth_stencil_texture: None,
                        texture_array_index: None,
                        viewport,
                    })
                    .collect(),
            })
            .collect();
        Some(Frame {
            pose: Some(ViewerPose {
                transform: RigidTransform3D::identity(),
                views: Views::Stereo(left, right),
            }),
            inputs: vec![],
            events: vec![],
            sub_images,
            hit_test_results: vec![],
            predicted_display_time: 16.0,
        })
    }

    fn end_animation_frame(&mut self, _layers: &[(ContextId, LayerId)]) {
        self.frames_ended.fetch_add(1, Ordering::SeqCst);
    }

    fn initial_inputs(&self) -> Vec<servo_webxr_api::InputSource> {
        vec![]
    }

    fn set_event_dest(&mut self, dest: ProfileGenericCallback<Event>) {
        self.events.upgrade(dest);
    }

    fn quit(&mut self) {
        self.quit_called.store(true, Ordering::SeqCst);
        self.events.callback(Event::SessionEnd);
    }

    fn set_quitter(&mut self, _quitter: Quitter) {}

    fn update_clip_planes(&mut self, near: f32, far: f32) {
        self.clip_planes.update(near, far);
    }

    fn environment_blend_mode(&self) -> EnvironmentBlendMode {
        EnvironmentBlendMode::Opaque
    }

    fn granted_features(&self) -> &[String] {
        &self.granted_features
    }

    fn update_frame_rate(&mut self, rate: f32) -> f32 {
        self.frame_rate = rate;
        self.frame_rate
    }

    fn supported_frame_rates(&self) -> Vec<f32> {
        vec![60.0, 90.0]
    }

    fn reference_space_bounds(&self) -> Option<Vec<Point2D<f32, Floor>>> {
        Some(vec![
            Point2D::new(-1.0, -1.0),
            Point2D::new(-1.0, 1.0),
            Point2D::new(1.0, 1.0),
            Point2D::new(1.0, -1.0),
        ])
    }
}

struct MockDiscovery {
    supported_mode: SessionMode,
    supported_features: Vec<String>,
    device_changed: Arc<AtomicBool>,
    requests: Arc<AtomicUsize>,
    frames_begun: Arc<AtomicUsize>,
    frames_ended: Arc<AtomicUsize>,
    quit_called: Arc<AtomicBool>,
}

impl DiscoveryAPI<()> for MockDiscovery {
    fn request_session(
        &mut self,
        mode: SessionMode,
        init: &SessionInit,
        xr: SessionBuilder<()>,
    ) -> Result<Session, Error> {
        if mode != self.supported_mode {
            return Err(Error::NoMatchingDevice);
        }
        self.requests.fetch_add(1, Ordering::SeqCst);
        let granted = init.validate(mode, &self.supported_features)?;
        let frames_begun = self.frames_begun.clone();
        let frames_ended = self.frames_ended.clone();
        let quit_called = self.quit_called.clone();
        xr.spawn(move |_| {
            Ok(MockDevice::new(
                granted,
                frames_begun,
                frames_ended,
                quit_called,
            ))
        })
    }

    fn supports_session(&self, mode: SessionMode) -> bool {
        mode == self.supported_mode
    }

    fn poll_device_changed(&mut self) -> bool {
        self.device_changed.swap(false, Ordering::SeqCst)
    }
}

fn wait_for_registry_reply<T: Send + 'static>(
    registry: &mut MainThreadRegistry<()>,
    rx: &mpsc::Receiver<T>,
) -> T {
    let deadline = Instant::now() + Duration::from_secs(5);
    while Instant::now() < deadline {
        registry.run_one_frame();
        if let Ok(val) = rx.try_recv() {
            return val;
        }
        std::thread::sleep(Duration::from_millis(2));
    }
    panic!("Timed out waiting for MainThreadRegistry reply");
}

#[test]
fn session_init_feature_validation() {
    let init = SessionInit {
        required_features: vec!["viewer".into(), "local".into(), "local-floor".into()],
        optional_features: vec!["bounded-floor".into(), "unknown-optional".into()],
        first_person_observer_view: false,
        painter_id: PainterId::next(),
    };
    let supported = vec!["local-floor".into(), "bounded-floor".into()];

    let granted = init
        .validate(SessionMode::ImmersiveVR, &supported)
        .expect("ImmersiveVR with supported features should validate");
    assert!(granted.contains(&"viewer".to_string()));
    assert!(granted.contains(&"local".to_string()));
    assert!(granted.contains(&"local-floor".to_string()));
    assert!(granted.contains(&"bounded-floor".to_string()));
    assert!(!granted.contains(&"unknown-optional".to_string()));

    // Inline sessions do not implicitly grant "local".
    let inline_init = SessionInit {
        required_features: vec!["local".into()],
        optional_features: vec![],
        first_person_observer_view: false,
        painter_id: PainterId::next(),
    };
    assert!(matches!(
        inline_init.validate(SessionMode::Inline, &[]),
        Err(Error::UnsupportedFeature(ref f)) if f == "local"
    ));

    // Unsupported required feature is rejected.
    let bad_init = SessionInit {
        required_features: vec!["hand-tracking".into()],
        optional_features: vec![],
        first_person_observer_view: false,
        painter_id: PainterId::next(),
    };
    assert!(matches!(
        bad_init.validate(SessionMode::ImmersiveVR, &supported),
        Err(Error::UnsupportedFeature(ref f)) if f == "hand-tracking"
    ));
}

#[test]
fn layer_init_texture_size_scaling() {
    let viewports = Viewports {
        viewports: vec![
            Rect::new(Point2D::new(0, 0), Size2D::new(800, 600)),
            Rect::new(Point2D::new(800, 0), Size2D::new(800, 600)),
        ],
    };

    let base_full = LayerInit::WebGLLayer {
        antialias: true,
        depth: true,
        stencil: false,
        alpha: true,
        ignore_depth_values: false,
        framebuffer_scale_factor: 1.0,
    };
    assert_eq!(base_full.texture_size(&viewports), Size2D::new(1600, 600));

    let base_half = LayerInit::WebGLLayer {
        antialias: true,
        depth: true,
        stencil: false,
        alpha: true,
        ignore_depth_values: false,
        framebuffer_scale_factor: 0.5,
    };
    assert_eq!(base_half.texture_size(&viewports), Size2D::new(800, 300));

    let proj = LayerInit::ProjectionLayer {
        depth: true,
        stencil: true,
        alpha: true,
        scale_factor: 1.5,
    };
    assert_eq!(proj.texture_size(&viewports), Size2D::new(2400, 900));
}

#[test]
fn main_thread_registry_primary_fallback_and_device_change() {
    let waker = MockWaker::default();
    let grand_manager = LayerGrandManager::new(NoopLayerGrandManager);
    let mut main_registry =
        MainThreadRegistry::new(Box::new(waker.clone()), grand_manager).unwrap();

    let primary_changed = Arc::new(AtomicBool::new(false));
    let primary_requests = Arc::new(AtomicUsize::new(0));
    let fallback_requests = Arc::new(AtomicUsize::new(0));

    main_registry.register(MockDiscovery {
        supported_mode: SessionMode::ImmersiveVR,
        supported_features: vec!["local-floor".into()],
        device_changed: primary_changed.clone(),
        requests: primary_requests.clone(),
        frames_begun: Default::default(),
        frames_ended: Default::default(),
        quit_called: Default::default(),
    });
    main_registry.register_fallback(MockDiscovery {
        supported_mode: SessionMode::Inline,
        supported_features: vec![],
        device_changed: Default::default(),
        requests: fallback_requests.clone(),
        frames_begun: Default::default(),
        frames_ended: Default::default(),
        quit_called: Default::default(),
    });

    let mut registry = main_registry.registry();

    // Primary supports ImmersiveVR; fallback supports Inline; neither supports ImmersiveAR.
    for (mode, expected_ok) in [
        (SessionMode::ImmersiveVR, true),
        (SessionMode::Inline, true),
        (SessionMode::ImmersiveAR, false),
    ] {
        let (tx, rx) = mpsc::channel();
        let cb = ProfileGenericCallback::new(move |res| {
            let _ = tx.send(res.unwrap());
        })
        .unwrap();
        registry.supports_session(mode, cb);
        let res = wait_for_registry_reply(&mut main_registry, &rx);
        assert_eq!(res.is_ok(), expected_ok, "unexpected support for {mode:?}");
    }

    // Device change callback fires when `poll_device_changed` returns true, and stops after removal.
    PipelineNamespace::install(PipelineNamespaceId(1));
    let pipeline_id = PipelineId::new();
    let change_count = Arc::new(AtomicUsize::new(0));
    let change_count_clone = change_count.clone();
    let change_cb = GenericCallback::new(move |_| {
        change_count_clone.fetch_add(1, Ordering::SeqCst);
    })
    .unwrap();
    registry.set_device_change_callback(pipeline_id, change_cb);
    main_registry.run_one_frame();
    assert_eq!(change_count.load(Ordering::SeqCst), 0);

    primary_changed.store(true, Ordering::SeqCst);
    main_registry.run_one_frame();
    assert_eq!(change_count.load(Ordering::SeqCst), 1);

    registry.remove_device_change_callbacks(pipeline_id);
    main_registry.run_one_frame();
    primary_changed.store(true, Ordering::SeqCst);
    main_registry.run_one_frame();
    assert_eq!(change_count.load(Ordering::SeqCst), 1);

    // Fallback handles Inline session request.
    let (tx, rx) = mpsc::channel();
    let cb = ProfileGenericCallback::new(move |res| {
        let _ = tx.send(res.unwrap());
    })
    .unwrap();
    registry.request_session(
        SessionMode::Inline,
        SessionInit {
            required_features: vec!["viewer".into()],
            optional_features: vec![],
            first_person_observer_view: false,
            painter_id: PainterId::next(),
        },
        cb,
    );
    let mut inline_session = wait_for_registry_reply(&mut main_registry, &rx).unwrap();
    assert_eq!(fallback_requests.load(Ordering::SeqCst), 1);
    assert_eq!(primary_requests.load(Ordering::SeqCst), 0);
    inline_session.end_session();
}

#[test]
fn session_thread_frame_loop_layers_and_clean_quit() {
    let waker = MockWaker::default();
    let grand_manager = LayerGrandManager::new(NoopLayerGrandManager);
    let mut main_registry = MainThreadRegistry::new(Box::new(waker), grand_manager).unwrap();

    let frames_begun = Arc::new(AtomicUsize::new(0));
    let frames_ended = Arc::new(AtomicUsize::new(0));
    let quit_called = Arc::new(AtomicBool::new(false));

    main_registry.register(MockDiscovery {
        supported_mode: SessionMode::ImmersiveVR,
        supported_features: vec!["local-floor".into(), "bounded-floor".into()],
        device_changed: Default::default(),
        requests: Default::default(),
        frames_begun: frames_begun.clone(),
        frames_ended: frames_ended.clone(),
        quit_called: quit_called.clone(),
    });

    let mut registry = main_registry.registry();
    let (tx, rx) = mpsc::channel();
    let cb = ProfileGenericCallback::new(move |res| {
        let _ = tx.send(res.unwrap());
    })
    .unwrap();
    registry.request_session(
        SessionMode::ImmersiveVR,
        SessionInit {
            required_features: vec!["local-floor".into()],
            optional_features: vec!["bounded-floor".into()],
            first_person_observer_view: false,
            painter_id: PainterId::next(),
        },
        cb,
    );
    let mut session = wait_for_registry_reply(&mut main_registry, &rx).unwrap();

    assert_eq!(session.viewports().len(), 2);
    assert_eq!(
        session.recommended_framebuffer_resolution(),
        Some(Size2D::<i32, Viewport>::new(1024, 512))
    );
    assert_eq!(
        session.environment_blend_mode(),
        EnvironmentBlendMode::Opaque
    );
    assert_eq!(session.supported_frame_rates(), &[60.0, 90.0]);
    assert_eq!(session.reference_space_bounds().unwrap().len(), 4);

    let (event_tx, event_rx) = mpsc::channel();
    let event_cb = ProfileGenericCallback::new(move |ev| {
        let _ = event_tx.send(ev.unwrap());
    })
    .unwrap();
    session.set_event_dest(event_cb);

    let (frame_tx, frame_rx) = mpsc::channel();
    let frame_cb = ProfileGenericCallback::new(move |frame| {
        let _ = frame_tx.send(frame.unwrap());
    })
    .unwrap();
    session.set_frame_dest(frame_cb);

    let context_id = ContextId(1);
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
    session.update_clip_planes(0.2, 500.0);

    let (rate_tx, rate_rx) = mpsc::channel();
    let rate_cb = ProfileGenericCallback::new(move |rate| {
        let _ = rate_tx.send(rate.unwrap());
    })
    .unwrap();
    session.update_frame_rate(60.0, rate_cb);
    assert_eq!(rate_rx.recv_timeout(Duration::from_secs(5)).unwrap(), 60.0);

    // Start render loop and pump 3 animation frames.
    session.start_render_loop();
    for _ in 0..3 {
        let frame = frame_rx.recv_timeout(Duration::from_secs(5)).unwrap();
        assert_eq!(frame.sub_images.len(), 1);
        assert_eq!(frame.sub_images[0].layer_id, layer_id);
        assert!(matches!(frame.pose.unwrap().views, Views::Stereo(_, _)));
        session.render_animation_frame();
    }
    let _final_frame = frame_rx.recv_timeout(Duration::from_secs(5)).unwrap();

    // Quit while in render loop; final render_animation_frame flushes end_frame then quits.
    session.end_session();
    session.render_animation_frame();

    let end_event = event_rx.recv_timeout(Duration::from_secs(5)).unwrap();
    assert!(matches!(end_event, Event::SessionEnd));
    assert!(quit_called.load(Ordering::SeqCst));
    assert_eq!(frames_begun.load(Ordering::SeqCst), 4);
    assert_eq!(frames_ended.load(Ordering::SeqCst), 4);
}

#[test]
fn fov_to_projection_matrix_upright_opengl_convention() {
    let mut clip = ClipPlanes::default();
    clip.update(0.1, 100.0);
    // Symmetric FOV with positive angle_up and negative angle_down (standard OpenXR).
    let m: Transform3D<f32, LeftEye, servo_webxr_api::Display> =
        fov_to_projection_matrix(-0.75, 0.75, 0.75, -0.75, clip);
    // Upright OpenGL/WebGL projection has positive X and Y scale factors.
    assert!(m.m11 > 0.0, "expected positive X scale, got {}", m.m11);
    assert!(m.m22 > 0.0, "expected positive Y scale, got {}", m.m22);
    assert!((m.m34 - (-1.0)).abs() < 1e-6);
}
