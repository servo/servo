/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// check-tidy: no specs after this line

use std::cell::OnceCell;
use std::rc::Rc;

use dom_struct::dom_struct;
use js::rust::HandleObject;
use net_traits::clientstorage::actors_child::ClientStorageTestCursorChild;
use script_bindings::inheritance::Castable;
use stylo_atoms::Atom;

use crate::dom::bindings::codegen::Bindings::ClientStorageTestCursorBinding::ClientStorageTestCursorMethods;
use crate::dom::bindings::num::Finite;
use crate::dom::bindings::refcounted::Trusted;
use crate::dom::bindings::reflector::{DomGlobal, reflect_dom_object_with_proto};
use crate::dom::bindings::root::DomRoot;
use crate::dom::event::{Event, EventBubbles, EventCancelable};
use crate::dom::eventtarget::EventTarget;
use crate::dom::globalscope::GlobalScope;
use crate::dom::progressevent::ProgressEvent;
use crate::script_runtime::CanGc;

#[dom_struct]
pub(crate) struct ClientStorageTestCursor {
    eventtarget: EventTarget,
    #[ignore_malloc_size_of = "Rc<T> is hard"]
    #[no_trace]
    child: OnceCell<Rc<ClientStorageTestCursorChild>>,
}

impl ClientStorageTestCursor {
    fn new_inherited() -> ClientStorageTestCursor {
        ClientStorageTestCursor {
            eventtarget: EventTarget::new_inherited(),
            child: OnceCell::new(),
        }
    }

    pub fn new(
        global: &GlobalScope,
        proto: Option<HandleObject>,
        can_gc: CanGc,
    ) -> DomRoot<ClientStorageTestCursor> {
        reflect_dom_object_with_proto(
            Box::new(ClientStorageTestCursor::new_inherited()),
            global,
            proto,
            can_gc,
        )
    }

    pub fn set_child(&self, child: Rc<ClientStorageTestCursorChild>) {
        let this = Trusted::new(self);

        child.set_response_callback(move |number| {
            let this = this.root();

            this.dispatch_response_event(number, CanGc::note());
        });

        let _ = self.child.set(child);
    }

    fn dispatch_response_event(&self, number: u64, can_gc: CanGc) {
        let progressevent = ProgressEvent::new(
            &self.global(),
            Atom::from("response"),
            EventBubbles::DoesNotBubble,
            EventCancelable::NotCancelable,
            false,
            Finite::wrap(number as f64),
            Finite::wrap(0 as f64),
            can_gc,
        );
        progressevent.upcast::<Event>().fire(self.upcast(), can_gc);
    }
}

impl ClientStorageTestCursorMethods<crate::DomTypeHolder> for ClientStorageTestCursor {
    fn Continue_(&self) {
        let child = self.child.get().unwrap();

        child.send_continue();
    }

    event_handler!(response, GetOnresponse, SetOnresponse);
}
