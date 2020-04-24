/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::reflector::{reflect_dom_object, Reflector};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::globalscope::GlobalScope;
use crate::dom::xrinputsource::XRInputSource;
use dom_struct::dom_struct;
use webxr_api::Hand;

#[dom_struct]
pub struct XRHand {
    reflector_: Reflector,
    #[ignore_malloc_size_of = "defined in webxr"]
    source: Dom<XRInputSource>,
    #[ignore_malloc_size_of = "partially defind in webxr"]
    support: Hand<()>,
}

impl XRHand {
    fn new_inherited(source: &XRInputSource, support: Hand<()>) -> XRHand {
        XRHand {
            reflector_: Reflector::new(),
            source: Dom::from_ref(source),
            support,
        }
    }

    pub fn new(global: &GlobalScope, source: &XRInputSource, support: Hand<()>) -> DomRoot<XRHand> {
        reflect_dom_object(Box::new(XRHand::new_inherited(source, support)), global)
    }
}
