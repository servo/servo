/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::rc::Rc;

use dom_struct::dom_struct;
use js::rust::HandleObject;

use crate::dom::bindings::codegen::Bindings::FontFaceSetBinding::FontFaceSetMethods;
use crate::dom::bindings::reflector::reflect_dom_object_with_proto;
use crate::dom::bindings::root::DomRoot;
use crate::dom::eventtarget::EventTarget;
use crate::dom::globalscope::GlobalScope;
use crate::dom::promise::Promise;
use crate::realms::enter_realm;

#[dom_struct]
pub struct FontFaceSet {
    target: EventTarget,
    #[ignore_malloc_size_of = "Rc"]
    promise: Rc<Promise>,
}

impl FontFaceSet {
    pub fn new_inherited(global: &GlobalScope) -> Self {
        FontFaceSet {
            target: EventTarget::new_inherited(),
            promise: Promise::new(global),
        }
    }

    pub fn new(global: &GlobalScope, proto: Option<HandleObject>) -> DomRoot<Self> {
        reflect_dom_object_with_proto(Box::new(FontFaceSet::new_inherited(global)), global, proto)
    }

    pub fn fulfill_ready_promise_if_needed(&self) {
        if !self.promise.is_fulfilled() {
            let _ac = enter_realm(&*self.promise);
            self.promise.resolve_native(self);
        }
    }
}

impl FontFaceSetMethods for FontFaceSet {
    /// https://drafts.csswg.org/css-font-loading/#dom-fontfaceset-ready
    fn Ready(&self) -> Rc<Promise> {
        self.promise.clone()
    }
}
