/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;

use base::generic_channel::GenericSend;
use base::id::ScriptEventLoopId;
use dom_struct::dom_struct;
use js::context::JSContext;
use js::conversions::ToJSValConvertible;
use js::jsapi::Heap;
use js::jsval::{DoubleValue, JSVal, ObjectValue, UndefinedValue};
use js::rust::HandleValue;
use profile_traits::generic_callback::GenericCallback;
use serde::{Deserialize, Serialize};
use storage_traits::indexeddb::{
    AsyncOperation, AsyncReadOnlyOperation, AsyncReadWriteOperation, BackendError, BackendResult,
    IndexedDBKeyType, IndexedDBRecord, IndexedDBThreadMsg, IndexedDBTxnMode, PutItemResult,
    SyncOperation,
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
            PutItemResult::Success => Self::None,
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
    fn send_request_handled(transaction: &IDBTransaction, request_id: u64, result: &'static str) {
        let global = transaction.global();
        let txn_id = transaction.get_serial_number();
        let db_name = transaction.get_db_name().to_string();
        println!(
            "[IDBDBG_REQ_HANDLED_SEND] txn={} req={} db={} result={}",
            txn_id, request_id, db_name, result
        );
        // https://w3c.github.io/IndexedDB/#transaction-lifecycle
        // Step 5:
        // The implementation must attempt to commit an inactive transaction when
        // all requests placed against the transaction have completed and their
        // returned results handled, no new requests have been placed against the
        // transaction, and the transaction has not been aborted
        // An explicit call to commit() will initiate a commit without waiting
        // for request results to be handled by script.
        // When committing, the transaction state is set to committing.
        // The implementation must atomically write any changes to the
        // database made by requests placed against the transaction.
        // That is, either all of the changes must be written, or if an error occurs,
        // such as a disk write error, the implementation must not write any of the changes
        // to the database, and the steps to abort a transaction will be followed.
        let send_result = global.storage_threads().send(IndexedDBThreadMsg::Sync(
            SyncOperation::RequestHandled {
                origin: global.origin().immutable().clone(),
                db_name: transaction.get_db_name().to_string(),
                txn: transaction.get_serial_number(),
                request_id,
            },
        ));
        if send_result.is_err() {
            eprintln!(
                "[IDBDBG_REQ_HANDLED_RESULT] txn={} req={} db={} result=error",
                txn_id, request_id, db_name
            );
            error!("Failed to send SyncOperation::RequestHandled");
        } else {
            println!(
                "[IDBDBG_REQ_HANDLED_RESULT] txn={} req={} db={} result=success",
                txn_id, request_id, db_name
            );
        }
        transaction.mark_request_handled(request_id);

        // Commit eligibility now that "handled" advanced.
        // This is the edge that was missing in your trace.
        transaction.maybe_commit();
    }

    // https://www.w3.org/TR/IndexedDB-2/#async-execute-request
    // Implements Step 5.4
    fn handle_async_request_finished(&self, cx: &mut JSContext, result: BackendResult<IdbResult>) {
        let request = self.request.root();
        let global = request.global();

        let transaction = request
            .transaction
            .get()
            .expect("Request unexpectedly has no transaction");
        let txn_id = transaction.get_serial_number();
        let db_name = transaction.get_db_name().to_string();
        let result_label = if result.is_ok() { "success" } else { "error" };
        let is_event_dispatch_test = db_name.contains("event-dispatch-active-flag");
        if is_event_dispatch_test {
            let event_loop_id = ScriptEventLoopId::installed();
            println!(
                "[IDBDBG_DELIVER_RECV] txn={} req={} db={} result={} event_loop={:?} active={} finished={} committing={}",
                txn_id,
                self.request_id,
                db_name,
                result_label,
                event_loop_id,
                transaction.is_active(),
                transaction.is_finished(),
                transaction.is_committing()
            );
            let found_req = transaction.has_request(&request);
            println!(
                "[IDBDBG_ROUTE_LOOKUP] txn={} req={} found_txn=true found_req={}",
                txn_id, self.request_id, found_req
            );
        }
        println!(
            "[IDBDBG_FINISH_ENTER] txn={} req={} db={} result={}",
            txn_id, self.request_id, db_name, result_label
        );

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
                        if is_event_dispatch_test {
                            println!(
                                "[IDBDBG_DELIVER_DROP] txn={} req={} db={} reason=structuredclone_error",
                                txn_id, self.request_id, db_name
                            );
                        }
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
                            if is_event_dispatch_test {
                                println!(
                                    "[IDBDBG_DELIVER_DROP] txn={} req={} db={} reason=structuredclone_error",
                                    txn_id, self.request_id, db_name
                                );
                            }
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
                            if is_event_dispatch_test {
                                println!(
                                    "[IDBDBG_DELIVER_DROP] txn={} req={} db={} reason=iterate_cursor_error",
                                    txn_id, self.request_id, db_name
                                );
                            }
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
                    if is_event_dispatch_test {
                        println!(
                            "[IDBDBG_DELIVER_DROP] txn={} req={} db={} reason=backend_error",
                            txn_id, self.request_id, db_name
                        );
                    }
                    Self::handle_async_request_error(&global, cx, request, error, self.request_id);
                    return;
                },
            }

            // Substep 3.1: Set the result of request to answer.
            request.set_result(answer.handle());

            // Substep 3.2: Set the error of request to undefined
            request.set_error(None, CanGc::from_cx(cx));

            // Substep 3.3: Fire a success event at request.
            // TODO: follow spec here
            let event = Event::new(
                &global,
                Atom::from("success"),
                EventBubbles::DoesNotBubble,
                EventCancelable::NotCancelable,
                CanGc::from_cx(cx),
            );

            // https://w3c.github.io/IndexedDB/#transaction-lifecycle
            // When each request associated with a transaction is processed,
            // a success or error event will be fired. While the event is being
            // dispatched, the transaction state is set to active, allowing additional
            // requests to be made against the transaction. Once the event dispatch is
            // complete, the transaction’s state is set to inactive again.
            transaction.set_active_flag(true);
            println!(
                "[IDBDBG_FINISH_BEFORE_FIRE] txn={} req={} db={} result=success",
                txn_id, self.request_id, db_name
            );
            event
                .upcast::<Event>()
                .fire(request.upcast(), CanGc::from_cx(cx));
            println!(
                "[IDBDBG_FINISH_AFTER_FIRE] txn={} req={} db={} result=success",
                txn_id, self.request_id, db_name
            );
            // https://w3c.github.io/IndexedDB/#transaction-lifecycle
            // Once the event dispatch is complete, the transaction’s state is set to inactive again.
            transaction.set_active_flag(false);
            println!(
                "[IDBDBG_FINISH_BEFORE_FINISHED] txn={} req={} db={} result=success",
                txn_id, self.request_id, db_name
            );
            if is_event_dispatch_test {
                println!(
                    "[IDBDBG_REQ_FINISH] txn={} req={} db={} result=success call_request_finished=true call_mark_handled=true",
                    txn_id, self.request_id, db_name
                );
            }
            // Notify the transaction that this request has finished.
            transaction.request_finished();
            println!(
                "[IDBDBG_FINISH_BEFORE_HANDLED] txn={} req={} db={} result=success",
                txn_id, self.request_id, db_name
            );
            // https://w3c.github.io/IndexedDB/#transaction-lifecycle
            // The implementation must attempt to commit an inactive transaction
            // when all requests placed against the transaction have completed and
            // their returned results handled, no new requests have been placed
            // against the transaction, and the transaction has not been aborted

            Self::send_request_handled(&transaction, self.request_id, "success");
        } else {
            // FIXME:(arihant2math) dispatch correct error
            // Substep 2
            if is_event_dispatch_test {
                println!(
                    "[IDBDBG_DELIVER_DROP] txn={} req={} db={} reason=backend_result_err",
                    txn_id, self.request_id, db_name
                );
            }
            Self::handle_async_request_error(
                &global,
                cx,
                request,
                Error::Data(None),
                self.request_id,
            );
        }
    }

    // https://www.w3.org/TR/IndexedDB-2/#async-execute-request
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
        let txn_id = transaction.get_serial_number();
        let db_name = transaction.get_db_name().to_string();
        let is_event_dispatch_test = db_name.contains("event-dispatch-active-flag");
        println!(
            "[IDBDBG_ERROR_ENTER] txn={} req={} db={} result=error",
            txn_id, request_id, db_name
        );
        // Substep 1: Set the result of request to undefined.
        rooted!(&in(cx) let undefined = UndefinedValue());
        request.set_result(undefined.handle());

        // Substep 2: Set the error of request to result.
        request.set_error(Some(error.clone()), CanGc::from_cx(cx));

        // Substep 3: Fire an error event at request.
        // TODO: follow the spec here
        let event = Event::new(
            global,
            Atom::from("error"),
            EventBubbles::Bubbles,
            EventCancelable::Cancelable,
            CanGc::from_cx(cx),
        );

        // https://w3c.github.io/IndexedDB/#transaction-lifecycle
        // When each request associated with a transaction is processed,
        // a success or error event will be fired. While the event is being
        // dispatched, the transaction state is set to active, allowing additional
        // requests to be made against the transaction. Once the event dispatch is
        // complete, the transaction’s state is set to inactive again.
        transaction.set_active_flag(true);
        println!(
            "[IDBDBG_ERROR_BEFORE_FIRE] txn={} req={} db={} result=error",
            txn_id, request_id, db_name
        );
        // https://w3c.github.io/IndexedDB/#events
        // Set event’s bubbles and cancelable attributes to false.
        let default_not_prevented = event
            .upcast::<Event>()
            .fire(request.upcast(), CanGc::from_cx(cx));
        println!(
            "[IDBDBG_ERROR_AFTER_FIRE] txn={} req={} db={} result=error",
            txn_id, request_id, db_name
        );
        // https://w3c.github.io/IndexedDB/#transaction-lifecycle
        // Once the event dispatch is complete, the transaction’s state is set to inactive again.
        transaction.set_active_flag(false);
        println!(
            "[IDBDBG_ERROR_BEFORE_ABORT] txn={} req={} db={} result=error default_not_prevented={}",
            txn_id, request_id, db_name, default_not_prevented
        );
        // https://w3c.github.io/IndexedDB/#transaction-lifecycle
        // An explicit call to abort() will initiate an abort. An abort will also be initiated following a failed request that is not handled by script.
        if default_not_prevented {
            // https://w3c.github.io/IndexedDB/#transaction-lifecycle
            // An explicit call to abort() will initiate an abort.
            // An abort will also be initiated following a failed request that is not handled by script.
            transaction.initiate_abort(error.clone(), CanGc::from_cx(cx));
            transaction.request_backend_abort();
        }
        println!(
            "[IDBDBG_ERROR_BEFORE_FINISHED] txn={} req={} db={} result=error",
            txn_id, request_id, db_name
        );
        if is_event_dispatch_test {
            println!(
                "[IDBDBG_REQ_FINISH] txn={} req={} db={} result=error call_request_finished=true call_mark_handled=true",
                txn_id, request_id, db_name
            );
        }
        // Notify the transaction that this request has finished.
        transaction.request_finished();
        println!(
            "[IDBDBG_ERROR_BEFORE_HANDLED] txn={} req={} db={} result=error",
            txn_id, request_id, db_name
        );
        // https://w3c.github.io/IndexedDB/#transaction-lifecycle
        // The implementation must attempt to commit an inactive transaction
        // when all requests placed against the transaction have completed and
        // their returned results handled, no new requests have been placed against
        // the transaction, and the transaction has not been aborted
        Self::send_request_handled(&transaction, request_id, "error");
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

    // https://www.w3.org/TR/IndexedDB-2/#asynchronously-execute-a-request
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
        let txn_id = transaction.get_serial_number();
        let db_name = transaction.get_db_name().to_string();

        // Step 2: Assert: transaction is active.
        if !transaction.is_active() || !transaction.is_usable() {
            return Err(Error::TransactionInactive(None));
        }

        let request_id = transaction.allocate_request_id();
        println!(
            "[IDBDBG_EXEC_ALLOC] txn={} req={} db={} result=pending",
            txn_id, request_id, db_name
        );

        // Step 3: If request was not given, let request be a new request with source as source.
        let request = request.unwrap_or_else(|| {
            let new_request = IDBRequest::new(&global, can_gc);
            new_request.set_source(Some(source));
            new_request.set_transaction(&transaction);
            new_request
        });

        // Step 4: Add request to the end of transaction’s request list.
        transaction.add_request(&request);
        println!(
            "[IDBDBG_EXEC_ADD] txn={} req={} db={} result=pending",
            txn_id, request_id, db_name
        );

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
            .dom_manipulation_task_source()
            .to_sendable();

        let cb_db_name = db_name.clone();
        let closure = move |message: Result<BackendResult<T>, ipc_channel::Error>| {
            println!(
                "[IDBDBG_ASYNC_CB_ENTER] txn={} req={} db={} result=pending",
                txn_id, request_id, cb_db_name
            );
            match &message {
                Ok(inner) => {
                    let result_label = if inner.is_ok() { "success" } else { "error" };
                    println!(
                        "[IDBDBG_ASYNC_CB_STATUS] txn={} req={} db={} result={}",
                        txn_id, request_id, cb_db_name, result_label
                    );
                },
                Err(err) => {
                    eprintln!(
                        "[IDBDBG_ASYNC_CB_STATUS] txn={} req={} db={} result=error err={:?}",
                        txn_id, request_id, cb_db_name, err
                    );
                },
            }
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

        if db_name.contains("event-dispatch-active-flag") {
            let op_label = match &operation {
                AsyncOperation::ReadOnly(op) => match op {
                    AsyncReadOnlyOperation::GetKey { .. } => "ReadOnly::GetKey",
                    AsyncReadOnlyOperation::GetItem { .. } => "ReadOnly::GetItem",
                    AsyncReadOnlyOperation::GetAllKeys { .. } => "ReadOnly::GetAllKeys",
                    AsyncReadOnlyOperation::GetAllItems { .. } => "ReadOnly::GetAllItems",
                    AsyncReadOnlyOperation::Count { .. } => "ReadOnly::Count",
                    AsyncReadOnlyOperation::Iterate { .. } => "ReadOnly::Iterate",
                },
                AsyncOperation::ReadWrite(op) => match op {
                    AsyncReadWriteOperation::PutItem { .. } => "ReadWrite::PutItem",
                    AsyncReadWriteOperation::RemoveItem { .. } => "ReadWrite::RemoveItem",
                    AsyncReadWriteOperation::Clear(_) => "ReadWrite::Clear",
                },
            };
            let js_req_ptr = (&*request) as *const IDBRequest;
            println!(
                "[IDBDBG_ROUTE_INSERT] txn={} req={} db={} op={} js_req_ptr={:p}",
                txn_id, request_id, db_name, op_label, js_req_ptr
            );
        }

        // Start is a backend database task (spec). Script does not model it with a
        // separate queued task, backend scheduling decides when requests begin.
        println!(
            "[IDBDBG_EXEC_SEND] txn={} req={} db={} result=pending",
            txn_id, request_id, db_name
        );
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
    /// <https://www.w3.org/TR/IndexedDB-2/#dom-idbrequest-result>
    fn Result(&self, _cx: SafeJSContext, mut val: js::rust::MutableHandle<'_, js::jsapi::Value>) {
        val.set(self.result.get());
    }

    /// <https://www.w3.org/TR/IndexedDB-2/#dom-idbrequest-error>
    fn GetError(&self) -> Option<DomRoot<DOMException>> {
        self.error.get()
    }

    /// <https://www.w3.org/TR/IndexedDB-2/#dom-idbrequest-source>
    fn GetSource(&self) -> Option<DomRoot<IDBObjectStore>> {
        self.source.get()
    }

    /// <https://www.w3.org/TR/IndexedDB-2/#dom-idbrequest-transaction>
    fn GetTransaction(&self) -> Option<DomRoot<IDBTransaction>> {
        self.transaction.get()
    }

    /// <https://www.w3.org/TR/IndexedDB-2/#dom-idbrequest-readystate>
    fn ReadyState(&self) -> IDBRequestReadyState {
        self.ready_state.get()
    }

    // https://www.w3.org/TR/IndexedDB-2/#dom-idbrequest-onsuccess
    event_handler!(success, GetOnsuccess, SetOnsuccess);

    // https://www.w3.org/TR/IndexedDB-2/#dom-idbrequest-onerror
    event_handler!(error, GetOnerror, SetOnerror);
}
