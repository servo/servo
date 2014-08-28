/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::TreeWalkerBinding;
use dom::bindings::codegen::Bindings::TreeWalkerBinding::TreeWalkerMethods;
use dom::bindings::global::GlobalRef;
use dom::bindings::js::{JSRef, Temporary};
use dom::bindings::utils::{Reflectable, Reflector, reflect_dom_object};

#[deriving(Encodable)]
pub struct TreeWalker {
    pub reflector_: Reflector
}

impl TreeWalker {
    pub fn new_inherited() -> TreeWalker {
        TreeWalker {
            reflector_: Reflector::new()
        }
    }

    pub fn new(global: &GlobalRef) -> Temporary<TreeWalker> {
        reflect_dom_object(box TreeWalker::new_inherited(), global, TreeWalkerBinding::Wrap)
    }
}

impl<'a> TreeWalkerMethods for JSRef<'a, TreeWalker> {
}

impl Reflectable for TreeWalker {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        &self.reflector_
    }
}
