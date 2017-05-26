/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::NavigatorBinding;
use dom::bindings::codegen::Bindings::NavigatorBinding::NavigatorMethods;
use dom::bindings::codegen::Bindings::VRBinding::VRBinding::VRMethods;
use dom::bindings::js::{MutNullableJS, Root};
use dom::bindings::reflector::{Reflector, DomObject, reflect_dom_object};
use dom::bindings::str::DOMString;
use dom::bluetooth::Bluetooth;
use dom::gamepadlist::GamepadList;
use dom::mimetypearray::MimeTypeArray;
use dom::navigatorinfo;
use dom::permissions::Permissions;
use dom::pluginarray::PluginArray;
use dom::promise::Promise;
use dom::serviceworkercontainer::ServiceWorkerContainer;
use dom::vr::VR;
use dom::window::Window;
use dom_struct::dom_struct;
use std::rc::Rc;

#[dom_struct]
pub struct Navigator {
    reflector_: Reflector,
    bluetooth: MutNullableJS<Bluetooth>,
    plugins: MutNullableJS<PluginArray>,
    mime_types: MutNullableJS<MimeTypeArray>,
    service_worker: MutNullableJS<ServiceWorkerContainer>,
    vr: MutNullableJS<VR>,
    gamepads: MutNullableJS<GamepadList>,
    permissions: MutNullableJS<Permissions>,
}

impl Navigator {
    fn new_inherited() -> Navigator {
        Navigator {
            reflector_: Reflector::new(),
            bluetooth: Default::default(),
            plugins: Default::default(),
            mime_types: Default::default(),
            service_worker: Default::default(),
            vr: Default::default(),
            gamepads: Default::default(),
            permissions: Default::default(),
        }
    }

    pub fn new(window: &Window) -> Root<Navigator> {
        reflect_dom_object(box Navigator::new_inherited(),
                           window,
                           NavigatorBinding::Wrap)
    }
}

impl NavigatorMethods for Navigator {
    // https://html.spec.whatwg.org/multipage/#dom-navigator-product
    fn Product(&self) -> DOMString {
        navigatorinfo::Product()
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
        navigatorinfo::UserAgent()
    }

    // https://html.spec.whatwg.org/multipage/#dom-navigator-appversion
    fn AppVersion(&self) -> DOMString {
        navigatorinfo::AppVersion()
    }

    // https://webbluetoothcg.github.io/web-bluetooth/#dom-navigator-bluetooth
    fn Bluetooth(&self) -> Root<Bluetooth> {
        self.bluetooth.or_init(|| Bluetooth::new(&self.global()))
    }

    // https://html.spec.whatwg.org/multipage/#navigatorlanguage
    fn Language(&self) -> DOMString {
        navigatorinfo::Language()
    }

    // https://html.spec.whatwg.org/multipage/#dom-navigator-plugins
    fn Plugins(&self) -> Root<PluginArray> {
        self.plugins.or_init(|| PluginArray::new(&self.global()))
    }

    // https://html.spec.whatwg.org/multipage/#dom-navigator-mimetypes
    fn MimeTypes(&self) -> Root<MimeTypeArray> {
        self.mime_types.or_init(|| MimeTypeArray::new(&self.global()))
    }

    // https://html.spec.whatwg.org/multipage/#dom-navigator-javaenabled
    fn JavaEnabled(&self) -> bool {
        false
    }

    // https://w3c.github.io/ServiceWorker/#navigator-service-worker-attribute
    fn ServiceWorker(&self) -> Root<ServiceWorkerContainer> {
        self.service_worker.or_init(|| {
            ServiceWorkerContainer::new(&self.global())
        })
    }

    // https://html.spec.whatwg.org/multipage/#dom-navigator-cookieenabled
    fn CookieEnabled(&self) -> bool {
        true
    }

    // https://www.w3.org/TR/gamepad/#navigator-interface-extension
    fn GetGamepads(&self) -> Root<GamepadList> {
        let root = self.gamepads.or_init(|| {
            GamepadList::new(&self.global(), &[])
        });

        let vr_gamepads = self.Vr().get_gamepads();
        root.add_if_not_exists(&vr_gamepads);
        // TODO: Add not VR related gamepads
        root
    }
    // https://w3c.github.io/permissions/#navigator-and-workernavigator-extension
    fn Permissions(&self) -> Root<Permissions> {
        self.permissions.or_init(|| Permissions::new(&self.global()))
    }

    // https://w3c.github.io/webvr/spec/1.1/#navigator-getvrdisplays-attribute
    #[allow(unrooted_must_root)]
    fn GetVRDisplays(&self) -> Rc<Promise> {
        self.Vr().GetDisplays()
    }
}

impl Navigator {
    pub fn Vr(&self) -> Root<VR> {
        self.vr.or_init(|| VR::new(&self.global()))
    }
}
