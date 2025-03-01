/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;
use std::convert::TryInto;
use std::sync::LazyLock;

use dom_struct::dom_struct;
use js::rust::MutableHandleValue;

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::NavigatorBinding::NavigatorMethods;
use crate::dom::bindings::codegen::Bindings::WindowBinding::Window_Binding::WindowMethods;
use crate::dom::bindings::reflector::{reflect_dom_object, DomGlobal, Reflector};
use crate::dom::bindings::root::{DomRoot, MutNullableDom};
use crate::dom::bindings::str::DOMString;
use crate::dom::bindings::utils::to_frozen_array;
#[cfg(feature = "bluetooth")]
use crate::dom::bluetooth::Bluetooth;
use crate::dom::gamepad::Gamepad;
use crate::dom::gamepadevent::GamepadEventType;
use crate::dom::mediadevices::MediaDevices;
use crate::dom::mediasession::MediaSession;
use crate::dom::mimetypearray::MimeTypeArray;
use crate::dom::navigatorinfo;
use crate::dom::permissions::Permissions;
use crate::dom::pluginarray::PluginArray;
use crate::dom::serviceworkercontainer::ServiceWorkerContainer;
#[cfg(feature = "webgpu")]
use crate::dom::webgpu::gpu::GPU;
use crate::dom::window::Window;
#[cfg(feature = "webxr")]
use crate::dom::xrsystem::XRSystem;
use crate::script_runtime::{CanGc, JSContext};

pub(super) fn hardware_concurrency() -> u64 {
    static CPUS: LazyLock<u64> = LazyLock::new(|| num_cpus::get().try_into().unwrap_or(1));

    *CPUS
}

#[dom_struct]
pub(crate) struct Navigator {
    reflector_: Reflector,
    #[cfg(feature = "bluetooth")]
    bluetooth: MutNullableDom<Bluetooth>,
    plugins: MutNullableDom<PluginArray>,
    mime_types: MutNullableDom<MimeTypeArray>,
    service_worker: MutNullableDom<ServiceWorkerContainer>,
    #[cfg(feature = "webxr")]
    xr: MutNullableDom<XRSystem>,
    mediadevices: MutNullableDom<MediaDevices>,
    /// <https://www.w3.org/TR/gamepad/#dfn-gamepads>
    gamepads: DomRefCell<Vec<MutNullableDom<Gamepad>>>,
    permissions: MutNullableDom<Permissions>,
    mediasession: MutNullableDom<MediaSession>,
    #[cfg(feature = "webgpu")]
    gpu: MutNullableDom<GPU>,
    /// <https://www.w3.org/TR/gamepad/#dfn-hasgamepadgesture>
    has_gamepad_gesture: Cell<bool>,
}

impl Navigator {
    fn new_inherited() -> Navigator {
        Navigator {
            reflector_: Reflector::new(),
            #[cfg(feature = "bluetooth")]
            bluetooth: Default::default(),
            plugins: Default::default(),
            mime_types: Default::default(),
            service_worker: Default::default(),
            #[cfg(feature = "webxr")]
            xr: Default::default(),
            mediadevices: Default::default(),
            gamepads: Default::default(),
            permissions: Default::default(),
            mediasession: Default::default(),
            #[cfg(feature = "webgpu")]
            gpu: Default::default(),
            has_gamepad_gesture: Cell::new(false),
        }
    }

    pub(crate) fn new(window: &Window, can_gc: CanGc) -> DomRoot<Navigator> {
        reflect_dom_object(Box::new(Navigator::new_inherited()), window, can_gc)
    }

    #[cfg(feature = "webxr")]
    pub(crate) fn xr(&self) -> Option<DomRoot<XRSystem>> {
        self.xr.get()
    }

    pub(crate) fn get_gamepad(&self, index: usize) -> Option<DomRoot<Gamepad>> {
        self.gamepads.borrow().get(index).and_then(|g| g.get())
    }

    pub(crate) fn set_gamepad(&self, index: usize, gamepad: &Gamepad, can_gc: CanGc) {
        if let Some(gamepad_to_set) = self.gamepads.borrow().get(index) {
            gamepad_to_set.set(Some(gamepad));
        }
        if self.has_gamepad_gesture.get() {
            gamepad.set_exposed(true);
            if self.global().as_window().Document().is_fully_active() {
                gamepad.notify_event(GamepadEventType::Connected, can_gc);
            }
        }
    }

    pub(crate) fn remove_gamepad(&self, index: usize) {
        if let Some(gamepad_to_remove) = self.gamepads.borrow_mut().get(index) {
            gamepad_to_remove.set(None);
        }
        self.shrink_gamepads_list();
    }

    /// <https://www.w3.org/TR/gamepad/#dfn-selecting-an-unused-gamepad-index>
    pub(crate) fn select_gamepad_index(&self) -> u32 {
        let mut gamepad_list = self.gamepads.borrow_mut();
        if let Some(index) = gamepad_list.iter().position(|g| g.get().is_none()) {
            index as u32
        } else {
            let len = gamepad_list.len();
            gamepad_list.resize_with(len + 1, Default::default);
            len as u32
        }
    }

    fn shrink_gamepads_list(&self) {
        let mut gamepad_list = self.gamepads.borrow_mut();
        for i in (0..gamepad_list.len()).rev() {
            if gamepad_list.get(i).is_none() {
                gamepad_list.remove(i);
            } else {
                break;
            }
        }
    }

    pub(crate) fn has_gamepad_gesture(&self) -> bool {
        self.has_gamepad_gesture.get()
    }

    pub(crate) fn set_has_gamepad_gesture(&self, has_gamepad_gesture: bool) {
        self.has_gamepad_gesture.set(has_gamepad_gesture);
    }
}

impl NavigatorMethods<crate::DomTypeHolder> for Navigator {
    // https://html.spec.whatwg.org/multipage/#dom-navigator-product
    fn Product(&self) -> DOMString {
        navigatorinfo::Product()
    }

    // https://html.spec.whatwg.org/multipage/#dom-navigator-productsub
    fn ProductSub(&self) -> DOMString {
        navigatorinfo::ProductSub()
    }

    // https://html.spec.whatwg.org/multipage/#dom-navigator-vendor
    fn Vendor(&self) -> DOMString {
        navigatorinfo::Vendor()
    }

    // https://html.spec.whatwg.org/multipage/#dom-navigator-vendorsub
    fn VendorSub(&self) -> DOMString {
        navigatorinfo::VendorSub()
    }

    // https://html.spec.whatwg.org/multipage/#dom-navigator-taintenabled
    fn TaintEnabled(&self) -> bool {
        navigatorinfo::TaintEnabled()
    }

    // https://html.spec.whatwg.org/multipage/#dom-navigator-appname
    fn AppName(&self) -> DOMString {
        navigatorinfo::AppName()
    }

    // https://html.spec.whatwg.org/multipage/#dom-navigator-appcodename
    fn AppCodeName(&self) -> DOMString {
        navigatorinfo::AppCodeName()
    }

    // https://html.spec.whatwg.org/multipage/#dom-navigator-platform
    fn Platform(&self) -> DOMString {
        navigatorinfo::Platform()
    }

    // https://html.spec.whatwg.org/multipage/#dom-navigator-useragent
    fn UserAgent(&self) -> DOMString {
        navigatorinfo::UserAgent(self.global().get_user_agent())
    }

    // https://html.spec.whatwg.org/multipage/#dom-navigator-appversion
    fn AppVersion(&self) -> DOMString {
        navigatorinfo::AppVersion()
    }

    // https://webbluetoothcg.github.io/web-bluetooth/#dom-navigator-bluetooth
    #[cfg(feature = "bluetooth")]
    fn Bluetooth(&self) -> DomRoot<Bluetooth> {
        self.bluetooth
            .or_init(|| Bluetooth::new(&self.global(), CanGc::note()))
    }

    // https://html.spec.whatwg.org/multipage/#navigatorlanguage
    fn Language(&self) -> DOMString {
        navigatorinfo::Language()
    }

    // https://html.spec.whatwg.org/multipage/#dom-navigator-languages
    #[allow(unsafe_code)]
    fn Languages(&self, cx: JSContext, retval: MutableHandleValue) {
        to_frozen_array(&[self.Language()], cx, retval)
    }

    // https://html.spec.whatwg.org/multipage/#dom-navigator-plugins
    fn Plugins(&self) -> DomRoot<PluginArray> {
        self.plugins
            .or_init(|| PluginArray::new(&self.global(), CanGc::note()))
    }

    // https://html.spec.whatwg.org/multipage/#dom-navigator-mimetypes
    fn MimeTypes(&self) -> DomRoot<MimeTypeArray> {
        self.mime_types
            .or_init(|| MimeTypeArray::new(&self.global(), CanGc::note()))
    }

    // https://html.spec.whatwg.org/multipage/#dom-navigator-javaenabled
    fn JavaEnabled(&self) -> bool {
        false
    }

    // https://w3c.github.io/ServiceWorker/#navigator-service-worker-attribute
    fn ServiceWorker(&self) -> DomRoot<ServiceWorkerContainer> {
        self.service_worker
            .or_init(|| ServiceWorkerContainer::new(&self.global(), CanGc::note()))
    }

    // https://html.spec.whatwg.org/multipage/#dom-navigator-cookieenabled
    fn CookieEnabled(&self) -> bool {
        true
    }

    /// <https://www.w3.org/TR/gamepad/#dom-navigator-getgamepads>
    fn GetGamepads(&self) -> Vec<Option<DomRoot<Gamepad>>> {
        let global = self.global();
        let window = global.as_window();
        let doc = window.Document();

        // TODO: Handle permissions policy once implemented
        if !doc.is_fully_active() || !self.has_gamepad_gesture.get() {
            return Vec::new();
        }

        self.gamepads.borrow().iter().map(|g| g.get()).collect()
    }
    // https://w3c.github.io/permissions/#navigator-and-workernavigator-extension
    fn Permissions(&self) -> DomRoot<Permissions> {
        self.permissions
            .or_init(|| Permissions::new(&self.global(), CanGc::note()))
    }

    /// <https://immersive-web.github.io/webxr/#dom-navigator-xr>
    #[cfg(feature = "webxr")]
    fn Xr(&self) -> DomRoot<XRSystem> {
        self.xr
            .or_init(|| XRSystem::new(self.global().as_window(), CanGc::note()))
    }

    /// <https://w3c.github.io/mediacapture-main/#dom-navigator-mediadevices>
    fn MediaDevices(&self) -> DomRoot<MediaDevices> {
        self.mediadevices
            .or_init(|| MediaDevices::new(&self.global(), CanGc::note()))
    }

    /// <https://w3c.github.io/mediasession/#dom-navigator-mediasession>
    fn MediaSession(&self) -> DomRoot<MediaSession> {
        self.mediasession.or_init(|| {
            // There is a single MediaSession instance per Pipeline
            // and only one active MediaSession globally.
            //
            // MediaSession creation can happen in two cases:
            //
            // - If content gets `navigator.mediaSession`
            // - If a media instance (HTMLMediaElement so far) starts playing media.
            let global = self.global();
            let window = global.as_window();
            MediaSession::new(window, CanGc::note())
        })
    }

    // https://gpuweb.github.io/gpuweb/#dom-navigator-gpu
    #[cfg(feature = "webgpu")]
    fn Gpu(&self) -> DomRoot<GPU> {
        self.gpu.or_init(|| GPU::new(&self.global(), CanGc::note()))
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-navigator-hardwareconcurrency>
    fn HardwareConcurrency(&self) -> u64 {
        hardware_concurrency()
    }
}
