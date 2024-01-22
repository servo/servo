/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::rust::HandleObject;

use crate::dom::bindings::codegen::Bindings::XRMediaBindingBinding::XRMediaBinding_Binding::XRMediaBindingMethods;
use crate::dom::bindings::codegen::Bindings::XRMediaBindingBinding::XRMediaLayerInit;
use crate::dom::bindings::error::{Error, Fallible};
use crate::dom::bindings::reflector::{reflect_dom_object_with_proto, Reflector};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::htmlvideoelement::HTMLVideoElement;
use crate::dom::window::Window;
use crate::dom::xrcylinderlayer::XRCylinderLayer;
use crate::dom::xrequirectlayer::XREquirectLayer;
use crate::dom::xrquadlayer::XRQuadLayer;
use crate::dom::xrsession::XRSession;

#[dom_struct]
pub struct XRMediaBinding {
    reflector: Reflector,
    session: Dom<XRSession>,
}

impl XRMediaBinding {
    pub fn new_inherited(session: &XRSession) -> XRMediaBinding {
        XRMediaBinding {
            reflector: Reflector::new(),
            session: Dom::from_ref(session),
        }
    }

    fn new(
        global: &Window,
        proto: Option<HandleObject>,
        session: &XRSession,
    ) -> DomRoot<XRMediaBinding> {
        reflect_dom_object_with_proto(
            Box::new(XRMediaBinding::new_inherited(session)),
            global,
            proto,
        )
    }

    #[allow(non_snake_case)]
    pub fn Constructor(
        global: &Window,
        proto: Option<HandleObject>,
        session: &XRSession,
    ) -> Fallible<DomRoot<XRMediaBinding>> {
        // Step 1.
        if session.is_ended() {
            return Err(Error::InvalidState);
        }

        // Step 2.
        if !session.is_immersive() {
            return Err(Error::InvalidState);
        }

        // Steps 3-5.
        Ok(XRMediaBinding::new(global, proto, session))
    }
}

impl XRMediaBindingMethods for XRMediaBinding {
    /// <https://immersive-web.github.io/layers/#dom-xrmediabinding-createquadlayer>
    fn CreateQuadLayer(
        &self,
        _: &HTMLVideoElement,
        _: &XRMediaLayerInit,
    ) -> Fallible<DomRoot<XRQuadLayer>> {
        // https://github.com/servo/servo/issues/27493
        Err(Error::NotSupported)
    }

    /// <https://immersive-web.github.io/layers/#dom-xrmediabinding-createcylinderlayer>
    fn CreateCylinderLayer(
        &self,
        _: &HTMLVideoElement,
        _: &XRMediaLayerInit,
    ) -> Fallible<DomRoot<XRCylinderLayer>> {
        // https://github.com/servo/servo/issues/27493
        Err(Error::NotSupported)
    }

    /// <https://immersive-web.github.io/layers/#dom-xrmediabinding-createequirectlayer>
    fn CreateEquirectLayer(
        &self,
        _: &HTMLVideoElement,
        _: &XRMediaLayerInit,
    ) -> Fallible<DomRoot<XREquirectLayer>> {
        // https://github.com/servo/servo/issues/27493
        Err(Error::NotSupported)
    }
}
