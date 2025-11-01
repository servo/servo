/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;
use std::convert::TryInto;
use std::ops::Deref;
use std::sync::{Arc, LazyLock, Mutex};

use dom_struct::dom_struct;
use headers::HeaderMap;
use http::header::{self, HeaderValue};
use js::rust::MutableHandleValue;
use net_traits::request::{
    CredentialsMode, Destination, RequestBuilder, RequestId, RequestMode,
    is_cors_safelisted_request_content_type,
};
use net_traits::{
    FetchMetadata, FetchResponseListener, NetworkError, ResourceFetchTiming, ResourceTimingType,
};
use servo_config::pref;
use servo_url::ServoUrl;

use crate::body::Extractable;
use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::NavigatorBinding::NavigatorMethods;
use crate::dom::bindings::codegen::Bindings::WindowBinding::Window_Binding::WindowMethods;
use crate::dom::bindings::codegen::Bindings::XMLHttpRequestBinding::BodyInit;
use crate::dom::bindings::error::{Error, Fallible};
use crate::dom::bindings::refcounted::Trusted;
use crate::dom::bindings::reflector::{DomGlobal, Reflector, reflect_dom_object};
use crate::dom::bindings::root::{DomRoot, MutNullableDom};
use crate::dom::bindings::str::{DOMString, USVString};
use crate::dom::bindings::utils::to_frozen_array;
#[cfg(feature = "bluetooth")]
use crate::dom::bluetooth::Bluetooth;
use crate::dom::clipboard::Clipboard;
use crate::dom::credentialmanagement::credentialscontainer::CredentialsContainer;
use crate::dom::csp::{GlobalCspReporting, Violation};
use crate::dom::gamepad::Gamepad;
use crate::dom::gamepad::gamepadevent::GamepadEventType;
use crate::dom::geolocation::Geolocation;
use crate::dom::globalscope::GlobalScope;
use crate::dom::mediadevices::MediaDevices;
use crate::dom::mediasession::MediaSession;
use crate::dom::mimetypearray::MimeTypeArray;
use crate::dom::navigatorinfo;
use crate::dom::performance::performanceresourcetiming::InitiatorType;
use crate::dom::permissions::Permissions;
use crate::dom::pluginarray::PluginArray;
use crate::dom::serviceworkercontainer::ServiceWorkerContainer;
use crate::dom::servointernals::ServoInternals;
#[cfg(feature = "webgpu")]
use crate::dom::webgpu::gpu::GPU;
use crate::dom::window::Window;
#[cfg(feature = "webxr")]
use crate::dom::xrsystem::XRSystem;
use crate::network_listener::{PreInvoke, ResourceTimingListener, submit_timing};
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
    credentials: MutNullableDom<CredentialsContainer>,
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
    clipboard: MutNullableDom<Clipboard>,
    #[cfg(feature = "webgpu")]
    gpu: MutNullableDom<GPU>,
    /// <https://www.w3.org/TR/gamepad/#dfn-hasgamepadgesture>
    has_gamepad_gesture: Cell<bool>,
    servo_internals: MutNullableDom<ServoInternals>,
}

impl Navigator {
    fn new_inherited() -> Navigator {
        Navigator {
            reflector_: Reflector::new(),
            #[cfg(feature = "bluetooth")]
            bluetooth: Default::default(),
            credentials: Default::default(),
            plugins: Default::default(),
            mime_types: Default::default(),
            service_worker: Default::default(),
            #[cfg(feature = "webxr")]
            xr: Default::default(),
            mediadevices: Default::default(),
            gamepads: Default::default(),
            permissions: Default::default(),
            mediasession: Default::default(),
            clipboard: Default::default(),
            #[cfg(feature = "webgpu")]
            gpu: Default::default(),
            has_gamepad_gesture: Cell::new(false),
            servo_internals: Default::default(),
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
    /// <https://html.spec.whatwg.org/multipage/#dom-navigator-product>
    fn Product(&self) -> DOMString {
        navigatorinfo::Product()
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-navigator-productsub>
    fn ProductSub(&self) -> DOMString {
        navigatorinfo::ProductSub()
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-navigator-vendor>
    fn Vendor(&self) -> DOMString {
        navigatorinfo::Vendor()
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-navigator-vendorsub>
    fn VendorSub(&self) -> DOMString {
        navigatorinfo::VendorSub()
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-navigator-taintenabled>
    fn TaintEnabled(&self) -> bool {
        navigatorinfo::TaintEnabled()
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-navigator-appname>
    fn AppName(&self) -> DOMString {
        navigatorinfo::AppName()
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-navigator-appcodename>
    fn AppCodeName(&self) -> DOMString {
        navigatorinfo::AppCodeName()
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-navigator-platform>
    fn Platform(&self) -> DOMString {
        navigatorinfo::Platform()
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-navigator-useragent>
    fn UserAgent(&self) -> DOMString {
        navigatorinfo::UserAgent(&pref!(user_agent))
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-navigator-appversion>
    fn AppVersion(&self) -> DOMString {
        navigatorinfo::AppVersion()
    }

    // https://webbluetoothcg.github.io/web-bluetooth/#dom-navigator-bluetooth
    #[cfg(feature = "bluetooth")]
    fn Bluetooth(&self) -> DomRoot<Bluetooth> {
        self.bluetooth
            .or_init(|| Bluetooth::new(&self.global(), CanGc::note()))
    }

    /// <https://www.w3.org/TR/credential-management-1/#framework-credential-management>
    fn Credentials(&self) -> DomRoot<CredentialsContainer> {
        self.credentials
            .or_init(|| CredentialsContainer::new(&self.global(), CanGc::note()))
    }

    /// <https://www.w3.org/TR/geolocation/#navigator_interface>
    fn Geolocation(&self) -> DomRoot<Geolocation> {
        Geolocation::new(&self.global(), CanGc::note())
    }

    /// <https://html.spec.whatwg.org/multipage/#navigatorlanguage>
    fn Language(&self) -> DOMString {
        navigatorinfo::Language()
    }

    // https://html.spec.whatwg.org/multipage/#dom-navigator-languages
    #[allow(unsafe_code)]
    fn Languages(&self, cx: JSContext, can_gc: CanGc, retval: MutableHandleValue) {
        to_frozen_array(&[self.Language()], cx, retval, can_gc)
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-navigator-online>
    fn OnLine(&self) -> bool {
        true
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-navigator-plugins>
    fn Plugins(&self) -> DomRoot<PluginArray> {
        self.plugins
            .or_init(|| PluginArray::new(&self.global(), CanGc::note()))
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-navigator-mimetypes>
    fn MimeTypes(&self) -> DomRoot<MimeTypeArray> {
        self.mime_types
            .or_init(|| MimeTypeArray::new(&self.global(), CanGc::note()))
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-navigator-javaenabled>
    fn JavaEnabled(&self) -> bool {
        false
    }

    /// <https://w3c.github.io/ServiceWorker/#navigator-service-worker-attribute>
    fn ServiceWorker(&self) -> DomRoot<ServiceWorkerContainer> {
        self.service_worker
            .or_init(|| ServiceWorkerContainer::new(&self.global(), CanGc::note()))
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-navigator-cookieenabled>
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
    /// <https://w3c.github.io/permissions/#navigator-and-workernavigator-extension>
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

    /// <https://w3c.github.io/clipboard-apis/#h-navigator-clipboard>
    fn Clipboard(&self) -> DomRoot<Clipboard> {
        self.clipboard
            .or_init(|| Clipboard::new(&self.global(), CanGc::note()))
    }

    /// <https://w3c.github.io/beacon/#sec-processing-model>
    fn SendBeacon(&self, url: USVString, data: Option<BodyInit>, can_gc: CanGc) -> Fallible<bool> {
        let global = self.global();
        // Step 1. Set base to this's relevant settings object's API base URL.
        let base = global.api_base_url();
        // Step 2. Set origin to this's relevant settings object's origin.
        let origin = global.origin().immutable().clone();
        // Step 3. Set parsedUrl to the result of the URL parser steps with url and base.
        // If the algorithm returns an error, or if parsedUrl's scheme is not "http" or "https",
        // throw a "TypeError" exception and terminate these steps.
        let Ok(url) = ServoUrl::parse_with_base(Some(&base), &url) else {
            return Err(Error::Type("Cannot parse URL".to_owned()));
        };
        if !matches!(url.scheme(), "http" | "https") {
            return Err(Error::Type("URL is not http(s)".to_owned()));
        }
        let mut request_body = None;
        // Step 4. Let headerList be an empty list.
        let mut headers = HeaderMap::with_capacity(1);
        // Step 5. Let corsMode be "no-cors".
        let mut cors_mode = RequestMode::NoCors;
        // Step 6. If data is not null:
        if let Some(data) = data {
            // Step 6.1. Set transmittedData and contentType to the result of extracting data's byte stream
            // with the keepalive flag set.
            let extracted_body = data.extract(&global, can_gc)?;
            // Step 6.2. If the amount of data that can be queued to be sent by keepalive enabled requests
            // is exceeded by the size of transmittedData (as defined in HTTP-network-or-cache fetch),
            // set the return value to false and terminate these steps.
            if let Some(total_bytes) = extracted_body.total_bytes {
                if total_bytes > 64 * 1024 {
                    return Ok(false);
                }
            }
            // Step 6.3. If contentType is not null:
            if let Some(content_type) = extracted_body.content_type.as_ref() {
                // Set corsMode to "cors".
                cors_mode = RequestMode::CorsMode;
                // If contentType value is a CORS-safelisted request-header value for the Content-Type header,
                // set corsMode to "no-cors".
                if is_cors_safelisted_request_content_type(content_type.as_bytes().deref()) {
                    cors_mode = RequestMode::NoCors;
                }
                // Append a Content-Type header with value contentType to headerList.
                //
                // We cannot use typed header insertion with `mime::Mime` parsing here,
                // since it lowercases `charset=UTF-8`: https://github.com/hyperium/mime/issues/116
                headers.insert(
                    header::CONTENT_TYPE,
                    HeaderValue::from_str(&content_type.str()).unwrap(),
                );
            }
            request_body = Some(extracted_body.into_net_request_body().0);
        }
        // Step 7.1. Let req be a new request, initialized as follows:
        let request = RequestBuilder::new(None, url.clone(), global.get_referrer())
            .mode(cors_mode)
            .destination(Destination::None)
            .policy_container(global.policy_container())
            .insecure_requests_policy(global.insecure_requests_policy())
            .has_trustworthy_ancestor_origin(global.has_trustworthy_ancestor_or_current_origin())
            .method(http::Method::POST)
            .body(request_body)
            .origin(origin)
            // TODO: Set keep-alive flag
            .credentials_mode(CredentialsMode::Include)
            .headers(headers);
        // Step 7.2. Fetch req.
        global.fetch(
            request,
            Arc::new(Mutex::new(BeaconFetchListener {
                url,
                global: Trusted::new(&global),
                resource_timing: ResourceFetchTiming::new(ResourceTimingType::None),
            })),
            global.task_manager().networking_task_source().into(),
        );
        // Step 7. Set the return value to true, return the sendBeacon() call,
        // and continue to run the following steps in parallel:
        Ok(true)
    }

    /// <https://servo.org/internal-no-spec>
    fn Servo(&self) -> DomRoot<ServoInternals> {
        self.servo_internals
            .or_init(|| ServoInternals::new(&self.global(), CanGc::note()))
    }
}

struct BeaconFetchListener {
    /// URL of this request.
    url: ServoUrl,
    /// Timing data for this resource.
    resource_timing: ResourceFetchTiming,
    /// The global object fetching the report uri violation
    global: Trusted<GlobalScope>,
}

impl FetchResponseListener for BeaconFetchListener {
    fn process_request_body(&mut self, _: RequestId) {}

    fn process_request_eof(&mut self, _: RequestId) {}

    fn process_response(
        &mut self,
        _: RequestId,
        fetch_metadata: Result<FetchMetadata, NetworkError>,
    ) {
        _ = fetch_metadata;
    }

    fn process_response_chunk(&mut self, _: RequestId, chunk: Vec<u8>) {
        _ = chunk;
    }

    fn process_response_eof(
        &mut self,
        _: RequestId,
        response: Result<ResourceFetchTiming, NetworkError>,
    ) {
        _ = response;
    }

    fn resource_timing_mut(&mut self) -> &mut ResourceFetchTiming {
        &mut self.resource_timing
    }

    fn resource_timing(&self) -> &ResourceFetchTiming {
        &self.resource_timing
    }

    fn submit_resource_timing(&mut self) {
        submit_timing(self, CanGc::note())
    }

    fn process_csp_violations(&mut self, _request_id: RequestId, violations: Vec<Violation>) {
        let global = self.resource_timing_global();
        global.report_csp_violations(violations, None, None);
    }
}

impl ResourceTimingListener for BeaconFetchListener {
    fn resource_timing_information(&self) -> (InitiatorType, ServoUrl) {
        (InitiatorType::Beacon, self.url.clone())
    }

    fn resource_timing_global(&self) -> DomRoot<GlobalScope> {
        self.global.root()
    }
}

impl PreInvoke for BeaconFetchListener {
    fn should_invoke(&self) -> bool {
        true
    }
}
