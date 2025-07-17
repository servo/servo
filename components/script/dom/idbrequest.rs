/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;

use constellation_traits::StructuredSerializedData;
use dom_struct::dom_struct;
use ipc_channel::router::ROUTER;
use js::jsapi::Heap;
use js::jsval::{JSVal, UndefinedValue};
use js::rust::HandleValue;
use net_traits::IpcSend;
use net_traits::indexeddb_thread::{
    AsyncOperation, IdbResult, IndexedDBThreadMsg, IndexedDBTxnMode,
};
use profile_traits::ipc;
use stylo_atoms::Atom;

use crate::dom::bindings::codegen::Bindings::IDBRequestBinding::{
    IDBRequestMethods, IDBRequestReadyState,
};
use crate::dom::bindings::codegen::Bindings::IDBTransactionBinding::IDBTransactionMode;
use crate::dom::bindings::error::{Error, Fallible};
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::refcounted::Trusted;
use crate::dom::bindings::reflector::{DomGlobal, reflect_dom_object};
use crate::dom::bindings::root::{DomRoot, MutNullableDom};
use crate::dom::bindings::structuredclone;
use crate::dom::domexception::{DOMErrorName, DOMException};
use crate::dom::event::{Event, EventBubbles, EventCancelable};
use crate::dom::eventtarget::EventTarget;
use crate::dom::globalscope::GlobalScope;
use crate::dom::idbobjectstore::IDBObjectStore;
use crate::dom::idbtransaction::IDBTransaction;
use crate::indexed_db::key_type_to_jsval;
use crate::realms::enter_realm;
use crate::script_runtime::{CanGc, JSContext as SafeJSContext};

#[derive(Clone)]
struct RequestListener {
    request: Trusted<IDBRequest>,
}

impl RequestListener {
    fn handle_async_request_finished(&self, result: Result<Option<IdbResult>, ()>) {
        let request = self.request.root();
        let global = request.global();
        let cx = GlobalScope::get_cx();

        request.set_ready_state_done();

        let _ac = enter_realm(&*request);
        rooted!(in(*cx) let mut answer = UndefinedValue());

        if let Ok(Some(data)) = result {
            match data {
                IdbResult::Key(key) => {
                    key_type_to_jsval(GlobalScope::get_cx(), &key, answer.handle_mut())
                },
                IdbResult::Data(serialized_data) => {
                    let data = StructuredSerializedData {
                        serialized: serialized_data,
                        ..Default::default()
                    };

                    if structuredclone::read(&global, data, answer.handle_mut()).is_err() {
                        warn!("Error reading structuredclone data");
                    }
                },
            }

            request.set_result(answer.handle());

            let transaction = request
                .transaction
                .get()
                .expect("Request unexpectedly has no transaction");

            let event = Event::new(
                &global,
                Atom::from("success"),
                EventBubbles::DoesNotBubble,
                EventCancelable::NotCancelable,
                CanGc::note(),
            );

            transaction.set_active_flag(true);
            event
                .upcast::<Event>()
                .fire(request.upcast(), CanGc::note());
            transaction.set_active_flag(false);
        } else {
            request.set_result(answer.handle());

            // FIXME:(rasviitanen)
            // Set the error of request to result

            let transaction = request
                .transaction
                .get()
                .expect("Request has no transaction");

            let event = Event::new(
                &global,
                Atom::from("error"),
                EventBubbles::Bubbles,
                EventCancelable::Cancelable,
                CanGc::note(),
            );

            transaction.set_active_flag(true);
            event
                .upcast::<Event>()
                .fire(request.upcast(), CanGc::note());
            transaction.set_active_flag(false);
        }
    }
}

#[dom_struct]
pub struct IDBRequest {
    eventtarget: EventTarget,
    #[ignore_malloc_size_of = "mozjs"]
    result: Heap<JSVal>,
    error: MutNullableDom<DOMException>,
    source: MutNullableDom<IDBObjectStore>,
    transaction: MutNullableDom<IDBTransaction>,
    ready_state: Cell<IDBRequestReadyState>,
}

impl IDBRequest {
    pub fn new_inherited() -> IDBRequest {
        IDBRequest {
            eventtarget: EventTarget::new_inherited(),

            result: Heap::default(),
            error: Default::default(),
            source: Default::default(),
            transaction: Default::default(),
            ready_state: Cell::new(IDBRequestReadyState::Pending),
        }
    }

    pub fn new(global: &GlobalScope, can_gc: CanGc) -> DomRoot<IDBRequest> {
        reflect_dom_object(Box::new(IDBRequest::new_inherited()), global, can_gc)
    }

    pub fn set_source(&self, source: Option<&IDBObjectStore>) {
        self.source.set(source);
    }

    pub fn set_ready_state_done(&self) {
        self.ready_state.set(IDBRequestReadyState::Done);
    }

    pub fn set_result(&self, result: HandleValue) {
        self.result.set(result.get());
    }

    pub fn set_error(&self, error: Error, can_gc: CanGc) {
        // FIXME:(rasviitanen) Support all error types
        if let Error::Version = error {
            self.error.set(Some(&DOMException::new(
                &self.global(),
                DOMErrorName::VersionError,
                can_gc,
            )));
        }
    }

    pub fn set_transaction(&self, transaction: &IDBTransaction) {
        self.transaction.set(Some(transaction));
    }

    // https://www.w3.org/TR/IndexedDB-2/#asynchronously-execute-a-request
    pub fn execute_async(
        source: &IDBObjectStore,
        operation: AsyncOperation,
        request: Option<DomRoot<IDBRequest>>,
        can_gc: CanGc,
    ) -> Fallible<DomRoot<IDBRequest>> {
        // Step 1: Let transaction be the transaction associated with source.
        let transaction = source.transaction().expect("Store has no transaction");
        let global = transaction.global();

        // Step 2: Assert: transaction is active.
        if !transaction.is_active() {
            return Err(Error::TransactionInactive);
        }

        // Step 3: If request was not given, let request be a new request with source as source.
        let request = request.unwrap_or_else(|| {
            let new_request = IDBRequest::new(&global, can_gc);
            new_request.set_source(Some(source));
            new_request.set_transaction(&transaction);
            new_request
        });

        // Step 4: Add request to the end of transaction’s request list.
        transaction.add_request(&request);

        // Step 5: Run the operation, and queue a returning task in parallel
        // the result will be put into `receiver`
        let transaction_mode = match transaction.get_mode() {
            IDBTransactionMode::Readonly => IndexedDBTxnMode::Readonly,
            IDBTransactionMode::Readwrite => IndexedDBTxnMode::Readwrite,
            IDBTransactionMode::Versionchange => IndexedDBTxnMode::Versionchange,
        };

        let (sender, receiver) =
            ipc::channel::<Result<Option<IdbResult>, ()>>(global.time_profiler_chan().clone())
                .unwrap();

        let response_listener = RequestListener {
            request: Trusted::new(&request),
        };

        let task_source = global
            .task_manager()
            .database_access_task_source()
            .to_sendable();

        ROUTER.add_typed_route(
            receiver.to_ipc_receiver(),
            Box::new(move |message| {
                let response_listener = response_listener.clone();
                task_source.queue(task!(request_callback: move || {
                    response_listener.handle_async_request_finished(
                        message.expect("Could not unwrap message"));
                }));
            }),
        );

        transaction
            .global()
            .resource_threads()
            .sender()
            .send(IndexedDBThreadMsg::Async(
                sender,
                global.origin().immutable().clone(),
                transaction.get_db_name().to_string(),
                source.get_name().to_string(),
                transaction.get_serial_number(),
                transaction_mode,
                operation,
            ))
            .unwrap();

        // Step 6
        Ok(request)
    }
}

impl IDBRequestMethods<crate::DomTypeHolder> for IDBRequest {
    // https://www.w3.org/TR/IndexedDB-2/#dom-idbrequest-result
    fn Result(&self, _cx: SafeJSContext, mut val: js::rust::MutableHandle<'_, js::jsapi::Value>) {
        val.set(self.result.get());
    }

    // https://www.w3.org/TR/IndexedDB-2/#dom-idbrequest-error
    fn GetError(&self) -> Option<DomRoot<DOMException>> {
        self.error.get()
    }

    // https://www.w3.org/TR/IndexedDB-2/#dom-idbrequest-source
    fn GetSource(&self) -> Option<DomRoot<IDBObjectStore>> {
        self.source.get()
    }

    // https://www.w3.org/TR/IndexedDB-2/#dom-idbrequest-transaction
    fn GetTransaction(&self) -> Option<DomRoot<IDBTransaction>> {
        self.transaction.get()
    }

    // https://www.w3.org/TR/IndexedDB-2/#dom-idbrequest-readystate
    fn ReadyState(&self) -> IDBRequestReadyState {
        self.ready_state.get()
    }

    // https://www.w3.org/TR/IndexedDB-2/#dom-idbrequest-onsuccess
    event_handler!(success, GetOnsuccess, SetOnsuccess);

    // https://www.w3.org/TR/IndexedDB-2/#dom-idbrequest-onerror
    event_handler!(error, GetOnerror, SetOnerror);
}
