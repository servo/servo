/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;

use dom_struct::dom_struct;
use js::jsapi::Heap;
use js::jsval::JSVal;

use crate::dom::bindings::codegen::Bindings::IDBRequestBinding::{
    IDBRequestMethods, IDBRequestReadyState,
};
use crate::dom::bindings::reflector::reflect_dom_object;
use crate::dom::bindings::root::{DomRoot, MutNullableDom};
use crate::dom::domexception::DOMException;
use crate::dom::eventtarget::EventTarget;
use crate::dom::globalscope::GlobalScope;
use crate::dom::indexeddb_next::idbobjectstore::IDBObjectStore;
use crate::dom::indexeddb_next::idbtransaction::IDBTransaction;
use crate::script_runtime::{CanGc, JSContext as SafeJSContext};

/// An "object" implementing the spec’s IDBRequest interface:
/// <https://w3c.github.io/IndexedDB/#request-api>.
///
/// The IDBRequest interface provides the means to access the result of
/// an asynchronous request to a database.
///
/// The IDBRequest struct has a remote counterpart in the backend, which
/// performs some of the steps defined by the corresponding spec algorithms.
#[dom_struct]
pub struct IDBRequest {
    eventtarget: EventTarget,

    /// <https://w3c.github.io/IndexedDB/#request-done-flag>
    done_flag: Cell<bool>,
    /// <https://w3c.github.io/IndexedDB/#request-source>
    source: MutNullableDom<IDBObjectStore>,
    /// <https://w3c.github.io/IndexedDB/#request-result>
    #[ignore_malloc_size_of = "mozjs"]
    result: Heap<JSVal>,
    /// <https://w3c.github.io/IndexedDB/#request-error>
    error: MutNullableDom<DOMException>,
    /// <https://w3c.github.io/IndexedDB/#request-transaction>
    transaction: MutNullableDom<IDBTransaction>,
}

impl IDBRequest {
    pub fn _new_inherited() -> IDBRequest {
        IDBRequest {
            eventtarget: EventTarget::new_inherited(),

            done_flag: Cell::new(false),
            source: Default::default(),
            result: Heap::default(),
            error: Default::default(),
            transaction: Default::default(),
        }
    }

    pub fn _new(global: &GlobalScope, can_gc: CanGc) -> DomRoot<IDBRequest> {
        reflect_dom_object(Box::new(IDBRequest::_new_inherited()), global, can_gc)
    }
}

#[expect(unused_doc_comments)]
impl IDBRequestMethods<crate::DomTypeHolder> for IDBRequest {
    /// <https://w3c.github.io/IndexedDB/#dom-idbrequest-result>
    fn Result(&self, _cx: SafeJSContext, mut val: js::rust::MutableHandle<'_, js::jsapi::Value>) {
        val.set(self.result.get());
    }

    /// <https://w3c.github.io/IndexedDB/#dom-idbrequest-error>
    fn GetError(&self) -> Option<DomRoot<DOMException>> {
        self.error.get()
    }

    /// <https://w3c.github.io/IndexedDB/#dom-idbrequest-source>
    fn GetSource(&self) -> Option<DomRoot<IDBObjectStore>> {
        self.source.get()
    }

    /// <https://w3c.github.io/IndexedDB/#dom-idbrequest-transaction>
    fn GetTransaction(&self) -> Option<DomRoot<IDBTransaction>> {
        self.transaction.get()
    }

    /// <https://w3c.github.io/IndexedDB/#dom-idbrequest-readystate>
    fn ReadyState(&self) -> IDBRequestReadyState {
        if self.done_flag.get() {
            IDBRequestReadyState::Done
        } else {
            IDBRequestReadyState::Pending
        }
    }

    /// <https://w3c.github.io/IndexedDB/#dom-idbrequest-onsuccess>
    event_handler!(success, GetOnsuccess, SetOnsuccess);

    /// <https://w3c.github.io/IndexedDB/#dom-idbrequest-onerror>
    event_handler!(error, GetOnerror, SetOnerror);
}
