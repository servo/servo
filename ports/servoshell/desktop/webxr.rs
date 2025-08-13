/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use servo::config::pref;
use servo::config::prefs::{self, Preferences};
use servo::webxr::WebXrRegistry;
use servo::webxr::glwindow::GlWindowDiscovery;
#[cfg(target_os = "windows")]
use servo::webxr::openxr::{AppInfo, OpenXrDiscovery};
use winit::event_loop::ActiveEventLoop;

use super::window_trait::WindowPortsMethods;

#[cfg(feature = "webxr")]
enum XrDiscovery {
    GlWindow(GlWindowDiscovery),
    #[cfg(target_os = "windows")]
    OpenXr(OpenXrDiscovery),
}

#[cfg(feature = "webxr")]
pub(crate) struct XrDiscoveryWebXrRegistry {
    xr_discovery: RefCell<Option<XrDiscovery>>,
}

impl XrDiscoveryWebXrRegistry {
    pub(crate) fn new_boxed(
        window: Rc<dyn WindowPortsMethods>,
        event_loop: Option<&ActiveEventLoop>,
        preferences: &Preferences,
    ) -> Box<Self> {
        let Some(event_loop) = event_loop else {
            return Box::new(Self {
                xr_discovery: RefCell::new(None),
            });
        };

        let xr_discovery = if preferences.dom_webxr_openxr_enabled {
            #[cfg(target_os = "windows")]
            {
                let app_info = AppInfo::new("Servoshell", 0, "Servo", 0);
                Some(XrDiscovery::OpenXr(OpenXrDiscovery::new(None, app_info)))
            }
            #[cfg(not(target_os = "windows"))]
            None
        } else if preferences.dom_webxr_glwindow_enabled {
            let window = window.new_glwindow(event_loop);
            Some(XrDiscovery::GlWindow(GlWindowDiscovery::new(window)))
        } else {
            None
        };

        Box::new(Self {
            xr_discovery: RefCell::new(xr_discovery),
        })
    }
}

struct XrPrefObserver(Arc<AtomicBool>);

impl prefs::Observer for XrPrefObserver {
    fn prefs_changed(&self, changes: Vec<(&'static str, prefs::PrefValue)>) {
        if let Some((_, value)) = changes.iter().find(|(name, _)| *name == "dom_webxr_test") {
            let prefs::PrefValue::Bool(value) = value else {
                return;
            };
            self.0.store(*value, Ordering::Relaxed);
        }
    }
}

#[cfg(feature = "webxr")]
impl WebXrRegistry for XrDiscoveryWebXrRegistry {
    fn register(&self, xr: &mut servo::webxr::MainThreadRegistry) {
        use servo::webxr::headless::HeadlessMockDiscovery;

        let mock_enabled = Arc::new(AtomicBool::new(pref!(dom_webxr_test)));
        xr.register_mock(HeadlessMockDiscovery::new(mock_enabled.clone()));
        prefs::set_observer(Box::new(XrPrefObserver(mock_enabled)));

        if let Some(xr_discovery) = self.xr_discovery.take() {
            match xr_discovery {
                XrDiscovery::GlWindow(discovery) => xr.register(discovery),
                #[cfg(target_os = "windows")]
                XrDiscovery::OpenXr(discovery) => xr.register(discovery),
            }
        }
    }
}
