/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// check-tidy: no specs after this line

use dom_struct::dom_struct;
use js::rust::HandleObject;

use crate::dom::bindings::codegen::Bindings::ClientStorageTestBinding::ClientStorageTestMethods;
use crate::dom::bindings::reflector::{Reflector, reflect_dom_object_with_proto};
use crate::dom::bindings::root::DomRoot;
use crate::dom::globalscope::GlobalScope;
use crate::script_runtime::CanGc;

#[dom_struct]
pub(crate) struct ClientStorageTest {
    reflector: Reflector,
}

impl ClientStorageTest {
    fn new_inherited() -> ClientStorageTest {
        ClientStorageTest {
            reflector: Reflector::new(),
        }
    }

    fn new(
        global: &GlobalScope,
        proto: Option<HandleObject>,
        can_gc: CanGc,
    ) -> DomRoot<ClientStorageTest> {
        reflect_dom_object_with_proto(
            Box::new(ClientStorageTest::new_inherited()),
            global,
            proto,
            can_gc,
        )
    }
}

impl ClientStorageTestMethods<crate::DomTypeHolder> for ClientStorageTest {
    fn Constructor(
        global: &GlobalScope,
        proto: Option<HandleObject>,
        can_gc: CanGc,
    ) -> DomRoot<ClientStorageTest> {
        ClientStorageTest::new(global, proto, can_gc)
    }

    fn Test(&self) -> i32 {
        42
    }
}
