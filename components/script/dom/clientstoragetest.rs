/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// check-tidy: no specs after this line

use std::cell::OnceCell;
use std::rc::Rc;

use dom_struct::dom_struct;
use js::rust::HandleObject;
use net_traits::clientstorage::actors_child::{
    ClientStorageTestChild, ClientStorageTestCursorChild,
};
use script_bindings::inheritance::Castable;
use stylo_atoms::Atom;

use crate::dom::bindings::codegen::Bindings::ClientStorageTestBinding::ClientStorageTestMethods;
use crate::dom::bindings::refcounted::Trusted;
use crate::dom::bindings::reflector::{DomGlobal, reflect_dom_object_with_proto};
use crate::dom::bindings::root::DomRoot;
use crate::dom::clientstoragetestcursor::ClientStorageTestCursor;
use crate::dom::event::{Event, EventBubbles, EventCancelable};
use crate::dom::eventtarget::EventTarget;
use crate::dom::globalscope::GlobalScope;
use crate::script_runtime::CanGc;

#[dom_struct]
pub(crate) struct ClientStorageTest {
    eventtarget: EventTarget,
    #[ignore_malloc_size_of = "Rc<T> is hard"]
    #[no_trace]
    child: OnceCell<Rc<ClientStorageTestChild>>,
}

impl ClientStorageTest {
    fn new_inherited() -> ClientStorageTest {
        ClientStorageTest {
            eventtarget: EventTarget::new_inherited(),
            child: OnceCell::new(),
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

    fn get_or_create_child(&self) -> Rc<ClientStorageTestChild> {
        if let Some(child) = self.child.get() {
            return Rc::clone(child);
        }

        let proxy = self.global().client_storage_proxy().unwrap();

        let child = ClientStorageTestChild::new();

        proxy.send_test_constructor(&child);

        let _ = self.child.set(Rc::clone(&child));

        child
    }

    fn dispatch_pong_event(&self, can_gc: CanGc) {
        let event = Event::new(
            &self.global(),
            Atom::from("pong"),
            EventBubbles::DoesNotBubble,
            EventCancelable::NotCancelable,
            can_gc,
        );
        event.fire(self.upcast(), can_gc);
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

    fn Ping(&self) {
        let child = self.get_or_create_child();

        let this = Trusted::new(self);

        child.set_pong_callback(move || {
            let this = this.root();

            this.dispatch_pong_event(CanGc::note());
        });

        child.send_ping();
    }

    fn OpenCursor(&self) -> DomRoot<ClientStorageTestCursor> {
        let cursor = ClientStorageTestCursor::new(&self.global(), None, CanGc::note());

        let cursor_child = ClientStorageTestCursorChild::new();

        let child = self.get_or_create_child();

        child.send_test_cursor_constructor(&cursor_child);

        cursor.set_child(cursor_child);

        cursor
    }

    event_handler!(pong, GetOnpong, SetOnpong);
}

impl Drop for ClientStorageTest {
    fn drop(&mut self) {
        if let Some(child) = self.child.get() {
            child.send_delete();
        }
    }
}
