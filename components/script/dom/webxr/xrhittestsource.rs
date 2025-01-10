/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use webxr_api::HitTestId;

use crate::dom::bindings::codegen::Bindings::XRHitTestSourceBinding::XRHitTestSourceMethods;
use crate::dom::bindings::reflector::{reflect_dom_object, Reflector};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::globalscope::GlobalScope;
use crate::dom::xrsession::XRSession;
use crate::script_runtime::CanGc;

#[dom_struct]
pub(crate) struct XRHitTestSource {
    reflector_: Reflector,
    #[ignore_malloc_size_of = "defined in webxr"]
    #[no_trace]
    id: HitTestId,
    session: Dom<XRSession>,
}

impl XRHitTestSource {
    fn new_inherited(id: HitTestId, session: &XRSession) -> XRHitTestSource {
        XRHitTestSource {
            reflector_: Reflector::new(),
            id,
            session: Dom::from_ref(session),
        }
    }

    pub(crate) fn new(
        global: &GlobalScope,
        id: HitTestId,
        session: &XRSession,
    ) -> DomRoot<XRHitTestSource> {
        reflect_dom_object(
            Box::new(XRHitTestSource::new_inherited(id, session)),
            global,
            CanGc::note(),
        )
    }

    pub(crate) fn id(&self) -> HitTestId {
        self.id
    }
}

impl XRHitTestSourceMethods<crate::DomTypeHolder> for XRHitTestSource {
    // https://immersive-web.github.io/hit-test/#dom-xrhittestsource-cancel
    fn Cancel(&self) {
        self.session.with_session(|s| s.cancel_hit_test(self.id));
    }
}
