/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;

use dom_struct::dom_struct;
use js::context::JSContext;
use js::conversions::ToJSValConvertible;
use js::jsapi::Heap;
use js::jsval::{DoubleValue, JSVal, ObjectValue, UndefinedValue};
use js::rust::HandleValue;
use profile_traits::generic_callback::GenericCallback;
use serde::{Deserialize, Serialize};
use servo_base::generic_channel::GenericSend;
use storage_traits::indexeddb::{
    AsyncOperation, AsyncReadOnlyOperation, BackendError, BackendResult, IndexedDBKeyType,
    IndexedDBRecord, IndexedDBThreadMsg, IndexedDBTxnMode, PutItemResult, SyncOperation,
};
use stylo_atoms::Atom;

use crate::dom::bindings::codegen::Bindings::IDBRequestBinding::{
    IDBRequestMethods, IDBRequestReadyState,
};
use crate::dom::bindings::codegen::Bindings::IDBTransactionBinding::IDBTransactionMode;
use crate::dom::bindings::error::{Error, Fallible, create_dom_exception};
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::refcounted::Trusted;
use crate::dom::bindings::reflector::{DomGlobal, DomObject, reflect_dom_object};
use crate::dom::bindings::root::{DomRoot, MutNullableDom};
use crate::dom::bindings::structuredclone;
use crate::dom::domexception::DOMException;
use crate::dom::event::{Event, EventBubbles, EventCancelable};
use crate::dom::eventtarget::EventTarget;
use crate::dom::globalscope::GlobalScope;
use crate::dom::indexeddb::idbcursor::{IterationParam, iterate_cursor};
use crate::dom::indexeddb::idbcursorwithvalue::IDBCursorWithValue;
use crate::dom::indexeddb::idbobjectstore::IDBObjectStore;
use crate::dom::indexeddb::idbtransaction::IDBTransaction;
use crate::indexeddb::key_type_to_jsval;
use crate::realms::enter_auto_realm;
use crate::script_runtime::{CanGc, JSContext as SafeJSContext};

#[derive(Clone)]
struct RequestListener {
    request: Trusted<IDBRequest>,
    iteration_param: Option<IterationParam>,
    request_id: u64,
}

pub enum IdbResult {
    Key(IndexedDBKeyType),
    Keys(Vec<IndexedDBKeyType>),
    Value(Vec<u8>),
    Values(Vec<Vec<u8>>),
    Count(u64),
    Iterate(Vec<IndexedDBRecord>),
    Error(Error),
    None,
}

impl From<IndexedDBKeyType> for IdbResult {
    fn from(value: IndexedDBKeyType) -> Self {
        IdbResult::Key(value)
    }
}

impl From<Vec<IndexedDBKeyType>> for IdbResult {
    fn from(value: Vec<IndexedDBKeyType>) -> Self {
        IdbResult::Keys(value)
    }
}

impl From<Vec<u8>> for IdbResult {
    fn from(value: Vec<u8>) -> Self {
        IdbResult::Value(value)
    }
}

impl From<Vec<Vec<u8>>> for IdbResult {
    fn from(value: Vec<Vec<u8>>) -> Self {
        IdbResult::Values(value)
    }
}

impl From<PutItemResult> for IdbResult {
    fn from(value: PutItemResult) -> Self {
        match value {
            PutItemResult::Key(key) => Self::Key(key),
            PutItemResult::CannotOverwrite => Self::Error(Error::Constraint(None)),
        }
    }
}

impl From<Vec<IndexedDBRecord>> for IdbResult {
    fn from(value: Vec<IndexedDBRecord>) -> Self {
        Self::Iterate(value)
    }
}

impl From<()> for IdbResult {
    fn from(_value: ()) -> Self {
        Self::None
    }
}

impl<T> From<Option<T>> for IdbResult
where
    T: Into<IdbResult>,
{
    fn from(value: Option<T>) -> Self {
        match value {
            Some(value) => value.into(),
            None => IdbResult::None,
        }
    }
}

impl From<u64> for IdbResult {
    fn from(value: u64) -> Self {
        IdbResult::Count(value)
    }
}

impl RequestListener {
    fn send_request_handled(transaction: &IDBTransaction, request_id: u64) {
        let global = transaction.global();
        // https://w3c.github.io/IndexedDB/#transaction-lifecycle
        // A transaction is inactive after control returns to the event loop and
        // when events are not being dispatched. We call this after dispatching
        // the request event, so the backend can reevaluate commit eligibility.
        let send_result = global.storage_threads().send(IndexedDBThreadMsg::Sync(
            SyncOperation::RequestHandled {
                origin: global.origin().immutable().clone(),
                db_name: transaction.get_db_name().to_string(),
                txn: transaction.get_serial_number(),
                request_id,
            },
        ));
        if send_result.is_err() {
            error!("Failed to send SyncOperation::RequestHandled");
        }
        transaction.mark_request_handled(request_id);

        // This request's result has been handled by script, the
        // transaction might finally be ready to auto-commit.
        transaction.maybe_commit();
    }

    // https://www.w3.org/TR/IndexedDB-3/#async-execute-request
    // Implements Step 5.4
    fn handle_async_request_finished(&self, cx: &mut JSContext, result: BackendResult<IdbResult>) {
        let request = self.request.root();
        let global = request.global();

        let transaction = request
            .transaction
            .get()
            .expect("Request unexpectedly has no transaction");
        // Substep 1: Set the result of request to result.
        request.set_ready_state_done();

        let mut realm = enter_auto_realm(cx, &*request);
        let cx: &mut JSContext = &mut realm;
        rooted!(&in(cx) let mut answer = UndefinedValue());

        if let Ok(data) = result {
            match data {
                IdbResult::Key(key) => key_type_to_jsval(cx, &key, answer.handle_mut()),
                IdbResult::Keys(keys) => {
                    rooted!(&in(cx) let mut array = vec![JSVal::default(); keys.len()]);
                    for (i, key) in keys.into_iter().enumerate() {
                        key_type_to_jsval(cx, &key, array.handle_mut_at(i));
                    }
                    array.safe_to_jsval(cx, answer.handle_mut());
                },
                IdbResult::Value(serialized_data) => {
                    let result = postcard::from_bytes(&serialized_data)
                        .map_err(|_| Error::Data(None))
                        .and_then(|data| {
                            structuredclone::read(
                                &global,
                                data,
                                answer.handle_mut(),
                                CanGc::from_cx(cx),
                            )
                        });
                    if let Err(e) = result {
                        warn!("Error reading structuredclone data");
                        Self::handle_async_request_error(&global, cx, request, e, self.request_id);
                        return;
                    };
                },
                IdbResult::Values(serialized_values) => {
                    rooted!(&in(cx) let mut values = vec![JSVal::default(); serialized_values.len()]);
                    for (i, serialized_data) in serialized_values.into_iter().enumerate() {
                        let result = postcard::from_bytes(&serialized_data)
                            .map_err(|_| Error::Data(None))
                            .and_then(|data| {
                                structuredclone::read(
                                    &global,
                                    data,
                                    values.handle_mut_at(i),
                                    CanGc::from_cx(cx),
                                )
                            });
                        if let Err(e) = result {
                            warn!("Error reading structuredclone data");
                            Self::handle_async_request_error(
                                &global,
                                cx,
                                request,
                                e,
                                self.request_id,
                            );
                            return;
                        };
                    }
                    values.safe_to_jsval(cx, answer.handle_mut());
                },
                IdbResult::Count(count) => {
                    answer.handle_mut().set(DoubleValue(count as f64));
                },
                IdbResult::Iterate(records) => {
                    let param = self.iteration_param.as_ref().expect(
                        "iteration_param must be provided by IDBRequest::execute_async for Iterate",
                    );
                    let cursor = match iterate_cursor(&global, cx, param, records) {
                        Ok(cursor) => cursor,
                        Err(e) => {
                            warn!("Error reading structuredclone data");
                            Self::handle_async_request_error(
                                &global,
                                cx,
                                request,
                                e,
                                self.request_id,
                            );
                            return;
                        },
                    };
                    if let Some(cursor) = cursor {
                        match cursor.downcast::<IDBCursorWithValue>() {
                            Some(cursor_with_value) => {
                                answer.handle_mut().set(ObjectValue(
                                    *cursor_with_value.reflector().get_jsobject(),
                                ));
                            },
                            None => {
                                answer
                                    .handle_mut()
                                    .set(ObjectValue(*cursor.reflector().get_jsobject()));
                            },
                        }
                    }
                },
                IdbResult::None => {
                    // no-op
                },
                IdbResult::Error(error) => {
                    // Substep 2
                    Self::handle_async_request_error(&global, cx, request, error, self.request_id);
                    return;
                },
            }

            // Substep 3.1: Set the result of request to answer.
            request.set_result(answer.handle());

            // Substep 3.2: Set the error of request to undefined
            request.set_error(None, CanGc::from_cx(cx));

            // https://w3c.github.io/IndexedDB/#fire-success-event
            // Step 1: Let event be the result of creating an event using Event.
            // Step 2: Set event’s type attribute to "success".
            // Step 3: Set event’s bubbles and cancelable attributes to false.
            let event = Event::new(
                &global,
                Atom::from("success"),
                EventBubbles::DoesNotBubble,
                EventCancelable::NotCancelable,
                CanGc::from_cx(cx),
            );

            // Step 5: Let legacyOutputDidListenersThrowFlag be initially false.
            let did_listeners_throw = Cell::new(false);
            // Step 6: If transaction’s state is inactive, then set transaction’s state to active.
            if transaction.is_inactive() {
                transaction.set_active_flag(true);
            }
            // Step 7: Dispatch event at request with legacyOutputDidListenersThrowFlag.
            event
                .upcast::<Event>()
                .fire_with_legacy_output_did_listeners_throw(
                    request.upcast(),
                    &did_listeners_throw,
                    CanGc::from_cx(cx),
                );
            // Step 8: If transaction’s state is active, then:
            if transaction.is_active() {
                // Step 8.1: Set transaction’s state to inactive.
                transaction.set_active_flag(false);
                // Step 8.2: If legacyOutputDidListenersThrowFlag is true, then run abort a
                // transaction with transaction and a newly created "AbortError" DOMException.
                if did_listeners_throw.get() {
                    transaction.initiate_abort(Error::Abort(None), CanGc::from_cx(cx));
                    transaction.request_backend_abort();
                }
            }
            transaction.request_finished();

            Self::send_request_handled(&transaction, self.request_id);
        } else {
            // FIXME:(arihant2math) dispatch correct error
            // Substep 2
            Self::handle_async_request_error(
                &global,
                cx,
                request,
                Error::Data(None),
                self.request_id,
            );
        }
    }

    // https://www.w3.org/TR/IndexedDB-3/#async-execute-request
    // Implements Step 5.4.2
    fn handle_async_request_error(
        global: &GlobalScope,
        cx: &mut JSContext,
        request: DomRoot<IDBRequest>,
        error: Error,
        request_id: u64,
    ) {
        let transaction = request
            .transaction
            .get()
            .expect("Request has no transaction");
        // Substep 1: Set the result of request to undefined.
        rooted!(&in(cx) let undefined = UndefinedValue());
        request.set_result(undefined.handle());

        // Substep 2: Set the error of request to result.
        request.set_error(Some(error.clone()), CanGc::from_cx(cx));

        // https://w3c.github.io/IndexedDB/#fire-error-event
        // Step 1: Let event be the result of creating an event using Event.
        // Step 2: Set event’s type attribute to "error".
        // Step 3: Set event’s bubbles and cancelable attributes to true.
        let event = Event::new(
            global,
            Atom::from("error"),
            EventBubbles::Bubbles,
            EventCancelable::Cancelable,
            CanGc::from_cx(cx),
        );

        // If result is an error and transaction’s state is committing, then run abort a
        // transaction with transaction and result, and terminate these steps.
        if transaction.is_committing() {
            transaction.initiate_abort(error.clone(), CanGc::from_cx(cx));
            transaction.request_backend_abort();
        }
        // Step 5: Let legacyOutputDidListenersThrowFlag be initially false.
        let did_listeners_throw = Cell::new(false);
        // Step 6: If transaction’s state is inactive, then set transaction’s state to active.
        if transaction.is_inactive() {
            transaction.set_active_flag(true);
        }
        // Step 7: Dispatch event at request with legacyOutputDidListenersThrowFlag.
        let default_not_prevented = event
            .upcast::<Event>()
            .fire_with_legacy_output_did_listeners_throw(
                request.upcast(),
                &did_listeners_throw,
                CanGc::from_cx(cx),
            );
        // Step 8: If transaction’s state is active, then:
        if transaction.is_active() {
            // Step 8.1: Set transaction’s state to inactive.
            transaction.set_active_flag(false);
            // Step 8.2: If legacyOutputDidListenersThrowFlag is true, then run abort a transaction
            // with transaction and a newly created "AbortError" DOMException and terminate these steps.
            // NOTE: This is done even if event’s canceled flag is false.
            // NOTE: This means that if an error event is fired and any of the event handlers throw an
            // exception, transaction’s error property is set to an AbortError rather than request’s
            // error, even if preventDefault() is never called.
            if did_listeners_throw.get() {
                transaction.initiate_abort(Error::Abort(None), CanGc::from_cx(cx));
                transaction.request_backend_abort();
            } else if default_not_prevented {
                // Step 8.3: If event’s canceled flag is false, then run abort a transaction
                // using transaction and request’s error, and terminate these steps.
                transaction.initiate_abort(error, CanGc::from_cx(cx));
                transaction.request_backend_abort();
            }
        }
        transaction.request_finished();
        Self::send_request_handled(&transaction, request_id);
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

    pub fn set_error(&self, error: Option<Error>, can_gc: CanGc) {
        if let Some(error) = error {
            if let Ok(exception) = create_dom_exception(&self.global(), error, can_gc) {
                self.error.set(Some(&exception));
            }
        } else {
            self.error.set(None);
        }
    }

    pub fn set_transaction(&self, transaction: &IDBTransaction) {
        self.transaction.set(Some(transaction));
    }

    pub fn clear_transaction(&self) {
        self.transaction.set(None);
    }

    fn is_done(&self) -> bool {
        self.ready_state.get() == IDBRequestReadyState::Done
    }

    pub(crate) fn transaction(&self) -> Option<DomRoot<IDBTransaction>> {
        self.transaction.get()
    }

    // https://www.w3.org/TR/IndexedDB-3/#asynchronously-execute-a-request
    pub fn execute_async<T, F>(
        source: &IDBObjectStore,
        operation_fn: F,
        request: Option<DomRoot<IDBRequest>>,
        iteration_param: Option<IterationParam>,
        can_gc: CanGc,
    ) -> Fallible<DomRoot<IDBRequest>>
    where
        T: Into<IdbResult> + for<'a> Deserialize<'a> + Serialize + Send + Sync + 'static,
        F: FnOnce(GenericCallback<BackendResult<T>>) -> AsyncOperation,
    {
        // Step 1: Let transaction be the transaction associated with source.
        let transaction = source.transaction();
        let global = transaction.global();
        // Step 2: Assert: transaction is active.
        if !transaction.is_active() || !transaction.is_usable() {
            return Err(Error::TransactionInactive(None));
        }

        let request_id = transaction.allocate_request_id();

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

        let response_listener = RequestListener {
            request: Trusted::new(&request),
            iteration_param: iteration_param.clone(),
            request_id,
        };

        let task_source = global
            .task_manager()
            .database_access_task_source()
            .to_sendable();

        let closure = move |message: Result<BackendResult<T>, ipc_channel::IpcError>| {
            let response_listener = response_listener.clone();
            task_source.queue(task!(request_callback: move |cx| {
                response_listener.handle_async_request_finished(
                    cx,
                    message.expect("Could not unwrap message").inspect_err(|e| {
                        if let BackendError::DbErr(e) = e {
                            error!("Error in IndexedDB operation: {}", e);
                        }
                    }).map(|t| t.into()),
                );
            }));
        };
        let callback = GenericCallback::new(global.time_profiler_chan().clone(), closure)
            .expect("Could not create callback");
        let operation = operation_fn(callback);

        if matches!(
            operation,
            AsyncOperation::ReadOnly(AsyncReadOnlyOperation::Iterate { .. })
        ) {
            assert!(
                iteration_param.is_some(),
                "iteration_param must be provided for Iterate"
            );
        } else {
            assert!(
                iteration_param.is_none(),
                "iteration_param should not be provided for operation other than Iterate"
            );
        }

        // Start is a backend database task (spec). Script does not model it with a
        // separate queued task, backend scheduling decides when requests begin.
        transaction
            .global()
            .storage_threads()
            .send(IndexedDBThreadMsg::Async(
                global.origin().immutable().clone(),
                transaction.get_db_name().to_string(),
                source.get_name().to_string(),
                transaction.get_serial_number(),
                request_id,
                transaction_mode,
                operation,
            ))
            .unwrap();

        // Step 6
        Ok(request)
    }
}

impl IDBRequestMethods<crate::DomTypeHolder> for IDBRequest {
    /// <https://www.w3.org/TR/IndexedDB-3/#dom-idbrequest-result>
    fn GetResult(
        &self,
        _cx: SafeJSContext,
        mut val: js::rust::MutableHandle<'_, js::jsapi::Value>,
    ) -> Fallible<()> {
        // Step 1. If this's done flag is false, then throw an "InvalidStateError" DOMException.
        if !self.is_done() {
            return Err(Error::InvalidState(Some(
                "Cannot get result on a request that is still pending.".into(),
            )));
        }

        // Step 2. Return this's result, or undefined if the request resulted in an error.
        val.set(self.result.get());
        Ok(())
    }

    /// <https://www.w3.org/TR/IndexedDB-3/#dom-idbrequest-error>
    fn GetError(&self) -> Fallible<Option<DomRoot<DOMException>>> {
        // Step 1. If this's done flag is false, then throw an "InvalidStateError" DOMException.
        if !self.is_done() {
            return Err(Error::InvalidState(Some(
                "Cannot get error on a request that is still pending.".into(),
            )));
        }

        // Step 2. Return this's error, or null if no error occurred.
        Ok(self.error.get())
    }

    /// <https://www.w3.org/TR/IndexedDB-3/#dom-idbrequest-source>
    fn GetSource(&self) -> Option<DomRoot<IDBObjectStore>> {
        self.source.get()
    }

    /// <https://www.w3.org/TR/IndexedDB-3/#dom-idbrequest-transaction>
    fn GetTransaction(&self) -> Option<DomRoot<IDBTransaction>> {
        self.transaction.get()
    }

    /// <https://www.w3.org/TR/IndexedDB-3/#dom-idbrequest-readystate>
    fn ReadyState(&self) -> IDBRequestReadyState {
        self.ready_state.get()
    }

    // https://www.w3.org/TR/IndexedDB-3/#dom-idbrequest-onsuccess
    event_handler!(success, GetOnsuccess, SetOnsuccess);

    // https://www.w3.org/TR/IndexedDB-3/#dom-idbrequest-onerror
    event_handler!(error, GetOnerror, SetOnerror);
}
