/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![cfg(all(feature = "openxr-api", target_os = "linux", not(target_env = "ohos")))]

use std::time::{Duration, Instant};

use embedder_traits::EventLoopWaker;
use profile_traits::generic_callback::GenericCallback as ProfileGenericCallback;
use servo_base::id::PainterId;
use servo_webxr::SurfmanGL;
use servo_webxr::openxr::{AppInfo, OpenXrDiscovery};
use webxr_api::{
    ContextId, DiscoveryAPI, Error, GLContexts, LayerGrandManager, LayerGrandManagerAPI, LayerId,
    LayerInit, LayerManager, LayerManagerAPI, LayerManagerFactory, MainThreadRegistry, SessionInit,
    SessionMode, SubImages,
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
struct UnusedLayerGrandManager;

impl LayerGrandManagerAPI<SurfmanGL> for UnusedLayerGrandManager {
    fn create_layer_manager(
        &self,
        _factory: LayerManagerFactory<SurfmanGL>,
    ) -> Result<LayerManager, Error> {
        Ok(LayerManager::new(UnusedLayerManager))
    }

    fn clone_layer_grand_manager(&self) -> LayerGrandManager<SurfmanGL> {
        LayerGrandManager::new(self.clone())
    }
}

struct UnusedLayerManager;

impl LayerManagerAPI<()> for UnusedLayerManager {
    fn create_layer(
        &mut self,
        _contexts: &mut dyn GLContexts<()>,
        _context_id: ContextId,
        _init: LayerInit,
    ) -> Result<LayerId, Error> {
        Err(Error::NoMatchingDevice)
    }

    fn destroy_layer(
        &mut self,
        _contexts: &mut dyn GLContexts<()>,
        _context_id: ContextId,
        _layer_id: LayerId,
    ) {
    }

    fn layers(&self) -> &[(ContextId, LayerId)] {
        &[]
    }

    fn begin_frame(
        &mut self,
        _contexts: &mut dyn GLContexts<()>,
        _layers: &[(ContextId, LayerId)],
    ) -> Result<Vec<SubImages>, Error> {
        Ok(vec![])
    }

    fn end_frame(
        &mut self,
        _contexts: &mut dyn GLContexts<()>,
        _layers: &[(ContextId, LayerId)],
    ) -> Result<(), Error> {
        Ok(())
    }
}

/// Runs in this dedicated integration-test binary so mutating `XR_RUNTIME_JSON`
/// cannot interfere with any other test process.
#[test]
fn linux_openxr_discovery_without_runtime_is_graceful() {
    // SAFETY: This integration test binary runs a single test that owns its process env.
    unsafe {
        std::env::set_var("XR_RUNTIME_JSON", "/nonexistent/servo-openxr-test.json");
    }

    let app_info = AppInfo::new("ServoOpenXrTest", 0, "Servo", 0);
    let discovery = OpenXrDiscovery::new(None, app_info);

    assert!(!discovery.supports_session(SessionMode::ImmersiveVR));
    assert!(!discovery.supports_session(SessionMode::ImmersiveAR));
    assert!(!discovery.supports_session(SessionMode::Inline));

    // Register into MainThreadRegistry and verify supports_session & request_session
    // return Error::NoMatchingDevice cleanly without panicking.
    let mut main_registry = MainThreadRegistry::new(
        Box::new(TestWaker),
        LayerGrandManager::new(UnusedLayerGrandManager),
    )
    .unwrap();
    main_registry.register(discovery);
    let mut registry = main_registry.registry();

    let (tx, rx) = crossbeam_channel::unbounded();
    registry.supports_session(
        SessionMode::ImmersiveVR,
        ProfileGenericCallback::new(move |res| {
            let _ = tx.send(res.unwrap());
        })
        .unwrap(),
    );
    let deadline = Instant::now() + Duration::from_secs(5);
    let support_res = loop {
        main_registry.run_one_frame();
        if let Ok(val) = rx.try_recv() {
            break val;
        }
        assert!(
            Instant::now() < deadline,
            "timed out waiting for supports_session"
        );
        std::thread::sleep(Duration::from_millis(2));
    };
    assert!(matches!(support_res, Err(Error::NoMatchingDevice)));

    let (tx, rx) = crossbeam_channel::unbounded();
    registry.request_session(
        SessionMode::ImmersiveVR,
        SessionInit {
            required_features: vec![],
            optional_features: vec![],
            first_person_observer_view: false,
            painter_id: PainterId::next(),
        },
        ProfileGenericCallback::new(move |res| {
            let _ = tx.send(res.unwrap());
        })
        .unwrap(),
    );
    let deadline = Instant::now() + Duration::from_secs(5);
    let req_res = loop {
        main_registry.run_one_frame();
        if let Ok(val) = rx.try_recv() {
            break val;
        }
        assert!(
            Instant::now() < deadline,
            "timed out waiting for request_session"
        );
        std::thread::sleep(Duration::from_millis(2));
    };
    assert!(matches!(req_res, Err(Error::NoMatchingDevice)));

    // Dropping MainThreadRegistry drops OpenXrDiscovery & OpenXRDevicePoller promptly.
    let drop_start = Instant::now();
    drop(main_registry);
    assert!(
        drop_start.elapsed() < Duration::from_secs(2),
        "dropping OpenXrDiscovery should shut down poller promptly"
    );
}
