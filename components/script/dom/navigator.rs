/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::compartments::InCompartment;
use crate::dom::bindings::codegen::Bindings::NavigatorBinding;
use crate::dom::bindings::codegen::Bindings::NavigatorBinding::NavigatorMethods;
use crate::dom::bindings::error::Error;
use crate::dom::bindings::reflector::{reflect_dom_object, DomObject, Reflector};
use crate::dom::bindings::root::{DomRoot, MutNullableDom};
use crate::dom::bindings::str::DOMString;
use crate::dom::bluetooth::Bluetooth;
use crate::dom::gamepadlist::GamepadList;
use crate::dom::gpu::GPU;
use crate::dom::identityhub::Identities;
use crate::dom::mediadevices::MediaDevices;
use crate::dom::mediasession::MediaSession;
use crate::dom::mimetypearray::MimeTypeArray;
use crate::dom::navigatorinfo;
use crate::dom::permissions::Permissions;
use crate::dom::pluginarray::PluginArray;
use crate::dom::promise::Promise;
use crate::dom::serviceworkercontainer::ServiceWorkerContainer;
use crate::dom::window::Window;
use crate::dom::xr::XR;
use dom_struct::dom_struct;
use smallvec::SmallVec;
use std::cell::RefCell;
use std::rc::Rc;
use webgpu::wgpu::{
    id::{AdapterId, BindGroupLayoutId, BufferId, DeviceId, PipelineLayoutId},
    Backend,
};

#[dom_struct]
pub struct Navigator {
    reflector_: Reflector,
    bluetooth: MutNullableDom<Bluetooth>,
    plugins: MutNullableDom<PluginArray>,
    mime_types: MutNullableDom<MimeTypeArray>,
    service_worker: MutNullableDom<ServiceWorkerContainer>,
    xr: MutNullableDom<XR>,
    mediadevices: MutNullableDom<MediaDevices>,
    gamepads: MutNullableDom<GamepadList>,
    permissions: MutNullableDom<Permissions>,
    mediasession: MutNullableDom<MediaSession>,
    gpu: MutNullableDom<GPU>,
    #[ignore_malloc_size_of = "Defined in wgpu"]
    gpu_id_hub: RefCell<Identities>,
}

impl Navigator {
    fn new_inherited() -> Navigator {
        Navigator {
            reflector_: Reflector::new(),
            bluetooth: Default::default(),
            plugins: Default::default(),
            mime_types: Default::default(),
            service_worker: Default::default(),
            xr: Default::default(),
            mediadevices: Default::default(),
            gamepads: Default::default(),
            permissions: Default::default(),
            mediasession: Default::default(),
            gpu: Default::default(),
            gpu_id_hub: RefCell::new(Identities::new()),
        }
    }

    pub fn new(window: &Window) -> DomRoot<Navigator> {
        reflect_dom_object(
            Box::new(Navigator::new_inherited()),
            window,
            NavigatorBinding::Wrap,
        )
    }
}

impl Navigator {
    pub fn create_adapter_ids(&self) -> SmallVec<[AdapterId; 4]> {
        self.gpu_id_hub.borrow_mut().create_adapter_ids()
    }

    pub fn create_device_id(&self, backend: Backend) -> DeviceId {
        self.gpu_id_hub.borrow_mut().create_device_id(backend)
    }

    pub fn create_buffer_id(&self, backend: Backend) -> BufferId {
        self.gpu_id_hub.borrow_mut().create_buffer_id(backend)
    }

    pub fn create_bind_group_layout_id(&self, backend: Backend) -> BindGroupLayoutId {
        self.gpu_id_hub
            .borrow_mut()
            .create_bind_group_layout_id(backend)
    }

    pub fn create_pipeline_layout_id(&self, backend: Backend) -> PipelineLayoutId {
        self.gpu_id_hub
            .borrow_mut()
            .create_pipeline_layout_id(backend)
    }
}

impl NavigatorMethods for Navigator {
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
    fn Bluetooth(&self) -> DomRoot<Bluetooth> {
        self.bluetooth.or_init(|| Bluetooth::new(&self.global()))
    }

    // https://html.spec.whatwg.org/multipage/#navigatorlanguage
    fn Language(&self) -> DOMString {
        navigatorinfo::Language()
    }

    // https://html.spec.whatwg.org/multipage/#dom-navigator-plugins
    fn Plugins(&self) -> DomRoot<PluginArray> {
        self.plugins.or_init(|| PluginArray::new(&self.global()))
    }

    // https://html.spec.whatwg.org/multipage/#dom-navigator-mimetypes
    fn MimeTypes(&self) -> DomRoot<MimeTypeArray> {
        self.mime_types
            .or_init(|| MimeTypeArray::new(&self.global()))
    }

    // https://html.spec.whatwg.org/multipage/#dom-navigator-javaenabled
    fn JavaEnabled(&self) -> bool {
        false
    }

    // https://w3c.github.io/ServiceWorker/#navigator-service-worker-attribute
    fn ServiceWorker(&self) -> DomRoot<ServiceWorkerContainer> {
        self.service_worker
            .or_init(|| ServiceWorkerContainer::new(&self.global()))
    }

    // https://html.spec.whatwg.org/multipage/#dom-navigator-cookieenabled
    fn CookieEnabled(&self) -> bool {
        true
    }

    // https://www.w3.org/TR/gamepad/#navigator-interface-extension
    fn GetGamepads(&self) -> DomRoot<GamepadList> {
        let root = self
            .gamepads
            .or_init(|| GamepadList::new(&self.global(), &[]));

        let vr_gamepads = self.Xr().get_gamepads();
        root.add_if_not_exists(&vr_gamepads);
        // TODO: Add not VR related gamepads
        root
    }
    // https://w3c.github.io/permissions/#navigator-and-workernavigator-extension
    fn Permissions(&self) -> DomRoot<Permissions> {
        self.permissions
            .or_init(|| Permissions::new(&self.global()))
    }

    // https://w3c.github.io/webvr/spec/1.1/#navigator-getvrdisplays-attribute
    fn GetVRDisplays(&self, comp: InCompartment) -> Rc<Promise> {
        let promise = Promise::new_in_current_compartment(&self.global(), comp);
        let displays = self.Xr().get_displays();
        match displays {
            Ok(displays) => promise.resolve_native(&displays),
            Err(_) => promise.reject_error(Error::Security),
        }
        promise
    }

    /// https://immersive-web.github.io/webxr/#dom-navigator-xr
    fn Xr(&self) -> DomRoot<XR> {
        self.xr.or_init(|| XR::new(&self.global()))
    }

    /// https://w3c.github.io/mediacapture-main/#dom-navigator-mediadevices
    fn MediaDevices(&self) -> DomRoot<MediaDevices> {
        self.mediadevices
            .or_init(|| MediaDevices::new(&self.global()))
    }

    /// https://w3c.github.io/mediasession/#dom-navigator-mediasession
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
            MediaSession::new(window)
        })
    }

    // https://gpuweb.github.io/gpuweb/#dom-navigator-gpu
    fn Gpu(&self) -> DomRoot<GPU> {
        self.gpu.or_init(|| GPU::new(&self.global()))
    }
}
