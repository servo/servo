/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// check-tidy: no specs after this line

use base::generic_channel::{self, GenericSend, GenericSender};
use dom_struct::dom_struct;
use js::rust::HandleObject;
use storage_traits::client_storage::{ClientStorageProxy, ClientStorageThreadMessage};

use crate::client_storage::StorageKeyConnection;
use crate::dom::bindings::codegen::Bindings::ClientStorageTestBinding::ClientStorageTestMethods;
use crate::dom::bindings::reflector::{Reflector, reflect_dom_object_with_proto};
use crate::dom::bindings::root::DomRoot;
use crate::dom::globalscope::GlobalScope;
use crate::script_runtime::CanGc;

#[dom_struct]
pub(crate) struct ClientStorageTest {
    reflector: Reflector,
    #[ignore_malloc_size_of = "Rc<T> is hard"]
    #[no_trace]
    connection: StorageKeyConnection,
}

impl ClientStorageTest {
    fn new_inherited(connection: StorageKeyConnection) -> ClientStorageTest {
        ClientStorageTest {
            reflector: Reflector::new(),
            connection,
        }
    }

    fn new(
        global: &GlobalScope,
        proto: Option<HandleObject>,
        can_gc: CanGc,
    ) -> DomRoot<ClientStorageTest> {
        let thread: GenericSender<ClientStorageThreadMessage> =
            GenericSend::sender(global.storage_threads());

        let proxy = ClientStorageProxy::new(thread);

        let origin = global.origin().immutable().clone();

        let connection = StorageKeyConnection::new(proxy, origin);

        reflect_dom_object_with_proto(
            Box::new(ClientStorageTest::new_inherited(connection)),
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
        let (sender, receiver) = generic_channel::channel().unwrap();
        self.connection.send_test(sender);
        receiver.recv().unwrap()
    }
}

impl Drop for ClientStorageTest {
    fn drop(&mut self) {
        debug!("Dropping script::ClientStorageTest");
    }
}
