/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::NavigatorBinding;
use dom::bindings::codegen::Bindings::NavigatorBinding::NavigatorMethods;
use dom::bindings::global::GlobalRef;
use dom::bindings::js::{JS, Root};
use dom::bindings::reflector::{Reflector, reflect_dom_object};
use dom::bluetooth::Bluetooth;
use dom::navigatorinfo;
use dom::window::Window;
use util::str::DOMString;

#[dom_struct]
pub struct Navigator {
    reflector_: Reflector,
    bluetooth: JS<Bluetooth>,
}

impl Navigator {
    fn new_inherited(window: &Window) -> Navigator {
        Navigator {
            reflector_: Reflector::new(),
            bluetooth: JS::from_ref(&Bluetooth::new(GlobalRef::Window(window))),
        }
    }

    pub fn new(window: &Window) -> Root<Navigator> {
        reflect_dom_object(box Navigator::new_inherited(&window),
                           GlobalRef::Window(window),
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
        Root::from_ref(&*self.bluetooth)
    }
}
