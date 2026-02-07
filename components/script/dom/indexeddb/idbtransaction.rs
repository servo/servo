/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;
use std::collections::{HashMap, HashSet};

use base::generic_channel::{GenericSend, GenericSender};
use base::id::ScriptEventLoopId;
use dom_struct::dom_struct;
use profile_traits::generic_callback::GenericCallback;
use profile_traits::generic_channel::channel;
use script_bindings::codegen::GenericUnionTypes::StringOrStringSequence;
use storage_traits::indexeddb::{
    IndexedDBIndex, IndexedDBThreadMsg, KeyPath, SyncOperation, TxnCompleteMsg,
};
use stylo_atoms::Atom;
use uuid::Uuid;

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::DOMStringListBinding::DOMStringListMethods;
use crate::dom::bindings::codegen::Bindings::IDBDatabaseBinding::IDBObjectStoreParameters;
use crate::dom::bindings::codegen::Bindings::IDBObjectStoreBinding::IDBIndexParameters;
use crate::dom::bindings::codegen::Bindings::IDBTransactionBinding::{
    IDBTransactionMethods, IDBTransactionMode,
};
use crate::dom::bindings::error::{Error, Fallible, create_dom_exception};
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::refcounted::Trusted;
use crate::dom::bindings::reflector::{DomGlobal, reflect_dom_object};
use crate::dom::bindings::root::{Dom, DomRoot, MutNullableDom};
use crate::dom::bindings::str::DOMString;
use crate::dom::domexception::DOMException;
use crate::dom::domstringlist::DOMStringList;
use crate::dom::event::{Event, EventBubbles, EventCancelable};
use crate::dom::eventtarget::EventTarget;
use crate::dom::globalscope::GlobalScope;
use crate::dom::indexeddb::idbdatabase::IDBDatabase;
use crate::dom::indexeddb::idbobjectstore::IDBObjectStore;
use crate::dom::indexeddb::idbrequest::IDBRequest;
use crate::script_runtime::CanGc;

#[dom_struct]
pub struct IDBTransaction {
    eventtarget: EventTarget,
    object_store_names: Dom<DOMStringList>,
    mode: IDBTransactionMode,
    db: Dom<IDBDatabase>,
    error: MutNullableDom<DOMException>,

    store_handles: DomRefCell<HashMap<String, Dom<IDBObjectStore>>>,
    // https://www.w3.org/TR/IndexedDB-2/#transaction-request-list
    requests: DomRefCell<Vec<Dom<IDBRequest>>>,
    // https://www.w3.org/TR/IndexedDB-2/#transaction-active-flag
    active: Cell<bool>,
    // https://www.w3.org/TR/IndexedDB-2/#transaction-finish
    finished: Cell<bool>,
    abort_initiated: Cell<bool>,
    abort_requested: Cell<bool>,
    committing: Cell<bool>,
    // https://w3c.github.io/IndexedDB/#transaction-concept
    // “A transaction optionally has a cleanup event loop which is an event loop.”
    #[no_trace]
    cleanup_event_loop: Cell<Option<ScriptEventLoopId>>,
    registered_in_global: Cell<bool>,
    // Tracks how many IDBRequest instances are still pending for this
    // transaction. The value is incremented when a request is added to the
    // transaction’s request list and decremented once the request has
    // finished.
    pending_request_count: Cell<usize>,
    next_request_id: Cell<u64>,
    handled_next_unhandled_id: Cell<u64>,
    handled_pending: DomRefCell<HashSet<u64>>,

    // An unique identifier, used to commit and revert this transaction
    // FIXME:(rasviitanen) Replace this with a channel
    serial_number: u64,

    /// The id of the associated open request, if any.
    #[no_trace]
    _open_request_id: Option<Uuid>,
}

impl IDBTransaction {
    fn new_inherited(
        connection: &IDBDatabase,
        mode: IDBTransactionMode,
        scope: &DOMStringList,
        serial_number: u64,
        open_request_id: Option<Uuid>,
    ) -> IDBTransaction {
        IDBTransaction {
            eventtarget: EventTarget::new_inherited(),
            object_store_names: Dom::from_ref(scope),
            mode,
            db: Dom::from_ref(connection),
            error: Default::default(),

            store_handles: Default::default(),
            requests: Default::default(),
            active: Cell::new(true),
            finished: Cell::new(false),
            abort_initiated: Cell::new(false),
            abort_requested: Cell::new(false),
            committing: Cell::new(false),
            cleanup_event_loop: Cell::new(None),
            registered_in_global: Cell::new(false),
            pending_request_count: Cell::new(0),
            next_request_id: Cell::new(0),
            handled_next_unhandled_id: Cell::new(0),
            handled_pending: Default::default(),
            serial_number,
            _open_request_id: open_request_id,
        }
    }

    /// Does a blocking call to get an id from the backend.
    /// TODO: remove in favor of something like `new_with_id` below.
    pub fn new(
        global: &GlobalScope,
        connection: &IDBDatabase,
        mode: IDBTransactionMode,
        scope: &DOMStringList,
        can_gc: CanGc,
    ) -> DomRoot<IDBTransaction> {
        let serial_number = IDBTransaction::register_new(global, connection.get_name());
        reflect_dom_object(
            Box::new(IDBTransaction::new_inherited(
                connection,
                mode,
                scope,
                serial_number,
                None,
            )),
            global,
            can_gc,
        )
    }

    /// Create a new WebIDL object,
    /// based on an existign transaction on the backend.
    /// The two are linked via the `transaction_id`.
    pub(crate) fn new_with_id(
        global: &GlobalScope,
        connection: &IDBDatabase,
        mode: IDBTransactionMode,
        scope: &DOMStringList,
        transaction_id: u64,
        open_request_id: Option<Uuid>,
        can_gc: CanGc,
    ) -> DomRoot<IDBTransaction> {
        reflect_dom_object(
            Box::new(IDBTransaction::new_inherited(
                connection,
                mode,
                scope,
                transaction_id,
                open_request_id,
            )),
            global,
            can_gc,
        )
    }

    // Registers a new transaction in the idb thread, and gets an unique serial number in return.
    // The serial number is used when placing requests against a transaction
    // and allows us to commit/abort transactions running in our idb thread.
    // FIXME:(rasviitanen) We could probably replace this with a channel instead,
    // and queue requests directly to that channel.
    fn register_new(global: &GlobalScope, db_name: DOMString) -> u64 {
        let (sender, receiver) = channel(global.time_profiler_chan().clone()).unwrap();

        global
            .storage_threads()
            .send(IndexedDBThreadMsg::Sync(SyncOperation::RegisterNewTxn(
                sender,
                global.origin().immutable().clone(),
                db_name.to_string(),
            )))
            .unwrap();

        receiver.recv().unwrap()
    }

    pub fn set_active_flag(&self, status: bool) {
        // https://w3c.github.io/IndexedDB/#transaction-lifecycle
        // “inactive … No requests can be made against the transaction when it is in this state.”
        // “finished … Once a transaction has committed or aborted, it enters this state.”
        // “When a transaction is committed or aborted, its state is set to finished.”
        if self.mode == IDBTransactionMode::Versionchange {
            println!(
                "IndexedDB versionchange set_active_flag: db={} txn={} active={}",
                self.db.get_name().to_string(),
                self.serial_number,
                status
            );
        }
        self.active.set(status);
    }

    pub fn is_active(&self) -> bool {
        self.active.get()
    }

    pub(crate) fn is_usable(&self) -> bool {
        // https://w3c.github.io/IndexedDB/#transaction-lifecycle
        // An abort will also be initiated following a failed request that is not handled by script.
        // A transaction can be aborted at any time before it is finished,
        // even if the transaction isn’t currently active or hasn’t yet started.

        // committing
        // Once all requests associated with a transaction have completed, the transaction will enter this state as it attempts to commit.
        // No requests can be made against the transaction when it is in this state.
        !self.finished.get() && !self.abort_initiated.get() && !self.committing.get()
    }

    pub(crate) fn is_finished(&self) -> bool {
        self.finished.get()
    }

    pub(crate) fn set_cleanup_event_loop(&self) {
        // https://w3c.github.io/IndexedDB/#transaction-concept
        // A transaction optionally has a cleanup event loop which is an event loop.
        self.cleanup_event_loop.set(ScriptEventLoopId::installed());
    }

    pub(crate) fn clear_cleanup_event_loop(&self) {
        // https://w3c.github.io/IndexedDB/#cleanup-indexed-database-transactions
        // Clear transaction’s cleanup event loop.
        self.cleanup_event_loop.set(None);
    }

    pub(crate) fn cleanup_event_loop_matches_current(&self) -> bool {
        match ScriptEventLoopId::installed() {
            Some(current) => self.cleanup_event_loop.get() == Some(current),
            None => false,
        }
    }

    pub(crate) fn set_registered_in_global(&self) {
        self.registered_in_global.set(true);
    }

    fn attempt_commit(&self) -> bool {
        println!(
            "IndexedDB attempt_commit called: db={} txn={} active={} committing={} finished={} abort_initiated={} pending_request_count={} handled_next_unhandled_id={} issued_count={}",
            self.db.get_name().to_string(),
            self.serial_number,
            self.active.get(),
            self.committing.get(),
            self.finished.get(),
            self.abort_initiated.get(),
            self.pending_request_count.get(),
            self.handled_next_unhandled_id.get(),
            self.issued_count()
        );
        let this = Trusted::new(self);
        let global = self.global();
        let task_source = global
            .task_manager()
            .dom_manipulation_task_source()
            .to_sendable();

        let callback = GenericCallback::new(
            global.time_profiler_chan().clone(),
            move |message: Result<TxnCompleteMsg, ipc_channel::Error>| {
                let this = this.clone();
                let task_source = task_source.clone();
                task_source.queue(task!(handle_commit_result: move || {
                    let this = this.root();
                    let message = message.expect("Could not unwrap message");
                    match message.result {
                        Ok(()) => {
                            println!(
                                "IndexedDB commit callback success: db={} txn={}",
                                this.db.get_name().to_string(),
                                this.serial_number
                            );
                            this.finalize_commit();
                        }
                        Err(_err) => {
                            println!(
                                "IndexedDB commit callback error; aborting txn: db={} txn={}",
                                this.db.get_name().to_string(),
                                this.serial_number
                            );
                            // TODO: Map backend commit error to appropriate DOMException.
                            this.initiate_abort(
                                Error::QuotaExceeded {
                                    quota: None,
                                    requested: None,
                                },
                                CanGc::note(),
                            );
                            this.finalize_abort();
                        }
                    }
                    // TODO: https://w3c.github.io/IndexedDB/#commit-a-transaction
                    // Backend commit/rollback is not yet atomic.
                }));
            },
        )
        .expect("Could not create callback");

        let commit_operation = SyncOperation::Commit(
            callback,
            global.origin().immutable().clone(),
            self.db.get_name().to_string(),
            self.serial_number,
        );

        // https://w3c.github.io/IndexedDB/#transaction-lifecycle
        // When committing, the transaction state is set to committing.
        // The implementation must atomically write any changes to the database made by requests
        // placed against the transaction. That is, either all of the changes must be written,
        // or if an error occurs, such as a disk write error, the implementation must not write
        // any of the changes to the database, and the steps to abort a transaction will be followed.
        println!(
            "IndexedDB sending SyncOperation::Commit: db={} txn={}",
            self.db.get_name().to_string(),
            self.serial_number
        );
        let send_result = self
            .get_idb_thread()
            .send(IndexedDBThreadMsg::Sync(commit_operation));
        if send_result.is_err() {
            println!(
                "IndexedDB failed to send SyncOperation::Commit: db={} txn={}",
                self.db.get_name().to_string(),
                self.serial_number
            );
            return false;
        }

        self.committing.set(true);
        println!(
            "IndexedDB marked transaction committing=true: db={} txn={}",
            self.db.get_name().to_string(),
            self.serial_number
        );
        true
    }

    pub(crate) fn maybe_commit(&self) {
        // https://w3c.github.io/IndexedDB/#transaction-lifecycle
        // “The implementation must attempt to commit an inactive transaction when all requests
        //  placed against the transaction have completed and their returned results handled,
        //  no new requests have been placed against the transaction, and the transaction has
        //  not been aborted.”
        let finished = self.finished.get();
        let abort_initiated = self.abort_initiated.get();
        let committing = self.committing.get();
        let active = self.active.get();
        let pending_request_count = self.pending_request_count.get();
        let handled_next_unhandled_id = self.handled_next_unhandled_id.get();
        let issued_count = self.issued_count();
        println!(
            "IndexedDB maybe_commit called: db={} txn={} finished={} abort_initiated={} committing={} active={} pending_request_count={} handled_next_unhandled_id={} issued_count={}",
            self.db.get_name().to_string(),
            self.serial_number,
            finished,
            abort_initiated,
            committing,
            active,
            pending_request_count,
            handled_next_unhandled_id,
            issued_count
        );
        if finished || abort_initiated || committing {
            println!(
                "IndexedDB maybe_commit early return (state gate): db={} txn={} finished={} abort_initiated={} committing={}",
                self.db.get_name().to_string(),
                self.serial_number,
                finished,
                abort_initiated,
                committing
            );
            return;
        }
        if active || pending_request_count != 0 {
            println!(
                "IndexedDB maybe_commit early return (active/pending gate): db={} txn={} active={} pending_request_count={}",
                self.db.get_name().to_string(),
                self.serial_number,
                active,
                pending_request_count
            );
            return;
        }
        if handled_next_unhandled_id != issued_count {
            println!(
                "IndexedDB maybe_commit early return (handled gate): db={} txn={} handled_next_unhandled_id={} issued_count={}",
                self.db.get_name().to_string(),
                self.serial_number,
                handled_next_unhandled_id,
                issued_count
            );
            return;
        }

        println!(
            "IndexedDB maybe_commit proceeding to attempt_commit: db={} txn={}",
            self.db.get_name().to_string(),
            self.serial_number
        );
        if !self.attempt_commit() {
            // We failed to initiate the commit algorithm (backend task could not be queued),
            // so the transaction cannot progress to a successful "complete".
            // Choose the most appropriate DOMException mapping for Servo here.
            self.initiate_abort(Error::InvalidState(None), CanGc::note());
            self.finalize_abort();
        }
    }

    fn force_commit(&self) {
        // https://w3c.github.io/IndexedDB/#transaction-lifecycle
        // An explicit call to commit() will initiate a commit without waiting for request results
        //  to be handled by script.
        //
        // This differs from automatic commit:
        // The implementation must attempt to commit an inactive transaction when all requests
        // placed against the transaction have completed and their returned results handled,
        // no new requests have been placed against the transaction, and the transaction has not been aborted
        if self.finished.get() || self.abort_initiated.get() || self.committing.get() {
            return;
        }
        if self.active.get() || self.pending_request_count.get() != 0 {
            return;
        }
        self.attempt_commit();
    }

    pub fn get_mode(&self) -> IDBTransactionMode {
        self.mode
    }

    pub fn get_db_name(&self) -> DOMString {
        self.db.get_name()
    }

    pub fn get_serial_number(&self) -> u64 {
        self.serial_number
    }

    pub(crate) fn issued_count(&self) -> u64 {
        self.next_request_id.get()
    }

    /// request_id is only required to be unique within this transaction.
    /// The backend keys “handled” state by (txn, request_id).
    pub(crate) fn allocate_request_id(&self) -> u64 {
        let id = self.next_request_id.get();
        self.next_request_id.set(id + 1);
        id
    }

    pub(crate) fn mark_request_handled(&self, request_id: u64) {
        let current = self.handled_next_unhandled_id.get();
        if request_id == current {
            let mut next = current + 1;
            {
                let mut pending = self.handled_pending.borrow_mut();
                while pending.remove(&next) {
                    next += 1;
                }
            }
            self.handled_next_unhandled_id.set(next);
        } else if request_id > current {
            self.handled_pending.borrow_mut().insert(request_id);
        }
    }

    pub fn add_request(&self, request: &IDBRequest) {
        self.requests.borrow_mut().push(Dom::from_ref(request));
        // Increase the number of outstanding requests so that we can detect when
        // the transaction is allowed to finish.
        self.pending_request_count
            .set(self.pending_request_count.get() + 1);
    }

    /// Must be called by an `IDBRequest` when it finishes (either success or
    /// error). This only updates the pending request count; `finished` is
    /// driven by backend commit/abort completion.
    pub fn request_finished(&self) {
        // https://w3c.github.io/IndexedDB/#transaction-lifecycle
        // finished
        // Once a transaction has committed or aborted, it enters this state.
        // No requests can be made against the transaction when it is in this state.
        if self.pending_request_count.get() == 0 {
            return;
        }
        let remaining = self.pending_request_count.get() - 1;
        self.pending_request_count.set(remaining);
    }

    pub(crate) fn initiate_abort(&self, error: Error, can_gc: CanGc) {
        // https://w3c.github.io/IndexedDB/#transaction-lifecycle
        // An abort will also be initiated following a failed request that is not handled by script.
        // A transaction can be aborted at any time before it is finished,
        // even if the transaction isn’t currently active or hasn’t yet started.
        if self.finished.get() || self.abort_initiated.get() {
            return;
        }
        self.abort_initiated.set(true);
        // https://w3c.github.io/IndexedDB/#transaction-concept
        // A transaction has a error which is set if the transaction is aborted.
        // NOTE: Implementors need to keep in mind that the value "null" is considered an error, as it is set from abort()
        if self.error.get().is_none() {
            if let Ok(exception) = create_dom_exception(&self.global(), error, can_gc) {
                self.error.set(Some(&exception));
            }
        }
    }

    pub(crate) fn request_backend_abort(&self) {
        if self.abort_requested.get() {
            return;
        }
        self.abort_requested.set(true);
        let this = Trusted::new(self);
        let global = self.global();
        let task_source = global
            .task_manager()
            .dom_manipulation_task_source()
            .to_sendable();
        let callback = GenericCallback::new(
            global.time_profiler_chan().clone(),
            move |message: Result<TxnCompleteMsg, ipc_channel::Error>| {
                let this = this.clone();
                let task_source = task_source.clone();
                task_source.queue(task!(handle_abort_result: move || {
                    let this = this.root();
                    let _ = message.expect("Could not unwrap message");
                    this.finalize_abort();
                }));
            },
        )
        .expect("Could not create callback");
        let operation = SyncOperation::Abort(
            callback,
            global.origin().immutable().clone(),
            self.db.get_name().to_string(),
            self.serial_number,
        );
        let _ = self
            .get_idb_thread()
            .send(IndexedDBThreadMsg::Sync(operation));
    }

    pub(crate) fn finalize_abort(&self) {
        if self.finished.get() {
            return;
        }
        println!(
            "IndexedDB finalize_abort: db={} txn={}",
            self.db.get_name().to_string(),
            self.serial_number
        );
        self.committing.set(false);
        let this = Trusted::new(self);
        self.global()
            .task_manager()
            .dom_manipulation_task_source()
            .queue(task!(send_abort_notification: move || {
                let this = this.root();
                this.active.set(false);
                if this.mode == IDBTransactionMode::Versionchange {
                    this.db.clear_upgrade_transaction(&this);
                }
                let global = this.global();
                let event = Event::new(
                    &global,
                    Atom::from("abort"),
                    EventBubbles::DoesNotBubble,
                    EventCancelable::NotCancelable,
                    CanGc::note(),
                );
                event.fire(this.upcast(), CanGc::note());
                println!(
                    "IndexedDB abort event fired: db={} txn={}",
                    this.db.get_name().to_string(),
                    this.serial_number
                );
                if this.mode == IDBTransactionMode::Versionchange {
                    let origin = this.global().origin().immutable().clone();
                    let db_name = this.db.get_name().to_string();
                    let txn = this.serial_number;
                    println!(
                        "IndexedDB sending UpgradeTransactionFinished committed=false: db={} txn={}",
                        this.db.get_name().to_string(),
                        this.serial_number
                    );
                    let _ = this.get_idb_thread().send(IndexedDBThreadMsg::Sync(
                        SyncOperation::UpgradeTransactionFinished {
                            origin,
                            db_name,
                            txn,
                            committed: false,
                        },
                    ));
                }
                // https://w3c.github.io/IndexedDB/#transaction-lifecycle
                // “When a transaction is committed or aborted, its state is set to finished.”
                this.finished.set(true);
                if this.registered_in_global.get() {
                    this.global().get_indexeddb().unregister_indexeddb_transaction(&this);
                    this.registered_in_global.set(false);
                }
            }));
    }

    pub(crate) fn finalize_commit(&self) {
        if self.finished.get() {
            return;
        }
        println!(
            "IndexedDB finalize_commit: db={} txn={}",
            self.db.get_name().to_string(),
            self.serial_number
        );
        self.committing.set(false);
        // https://w3c.github.io/IndexedDB/#transaction-lifecycle
        // When a transaction is committed or aborted, its state is set to finished.
        self.finished.set(true);
        if self.mode == IDBTransactionMode::Versionchange {
            self.db.clear_upgrade_transaction(self);
        }
        self.dispatch_complete();
        if self.registered_in_global.get() {
            self.global()
                .get_indexeddb()
                .unregister_indexeddb_transaction(self);
            self.registered_in_global.set(false);
        }
    }

    fn dispatch_complete(&self) {
        println!(
            "IndexedDB dispatch_complete queued: db={} txn={}",
            self.db.get_name().to_string(),
            self.serial_number
        );
        let global = self.global();
        let this = Trusted::new(self);
        global.task_manager().dom_manipulation_task_source().queue(
            task!(send_complete_notification: move || {
                let this = this.root();
                println!(
                    "IndexedDB dispatch_complete running: db={} txn={}",
                    this.db.get_name().to_string(),
                    this.serial_number
                );
                let global = this.global();
                let event = Event::new(
                    &global,
                    Atom::from("complete"),
                    EventBubbles::DoesNotBubble,
                    EventCancelable::NotCancelable,
                    CanGc::note()
                );
                event.fire(this.upcast(), CanGc::note());
                println!(
                    "IndexedDB complete event fired: db={} txn={}",
                    this.db.get_name().to_string(),
                    this.serial_number
                );
                if this.mode == IDBTransactionMode::Versionchange {
                    let origin = this.global().origin().immutable().clone();
                    let db_name = this.db.get_name().to_string();
                    let txn = this.serial_number;
                    println!(
                        "IndexedDB sending UpgradeTransactionFinished committed=true: db={} txn={}",
                        this.db.get_name().to_string(),
                        this.serial_number
                    );
                    let _ = this.get_idb_thread().send(IndexedDBThreadMsg::Sync(
                        SyncOperation::UpgradeTransactionFinished {
                            origin,
                            db_name,
                            txn,
                            committed: true,
                        },
                    ));
                }
            }),
        );
    }

    fn get_idb_thread(&self) -> GenericSender<IndexedDBThreadMsg> {
        self.global().storage_threads().sender()
    }

    fn object_store_parameters(
        &self,
        object_store_name: &DOMString,
    ) -> Option<(IDBObjectStoreParameters, Vec<IndexedDBIndex>)> {
        let global = self.global();
        let idb_sender = global.storage_threads().sender();
        let (sender, receiver) =
            channel(global.time_profiler_chan().clone()).expect("failed to create channel");

        let origin = global.origin().immutable().clone();
        let db_name = self.db.get_name().to_string();
        let object_store_name = object_store_name.to_string();

        let operation = SyncOperation::GetObjectStore(
            sender,
            origin.clone(),
            db_name.clone(),
            object_store_name.clone(),
        );

        let _ = idb_sender.send(IndexedDBThreadMsg::Sync(operation));

        // First unwrap for ipc
        // Second unwrap will never happen unless this db gets manually deleted somehow
        let object_store = receiver.recv().ok()?.ok()?;

        // First unwrap for ipc
        // Second unwrap will never happen unless this db gets manually deleted somehow
        let key_path = object_store.key_path.map(|key_path| match key_path {
            KeyPath::String(s) => StringOrStringSequence::String(DOMString::from_string(s)),
            KeyPath::Sequence(seq) => StringOrStringSequence::StringSequence(
                seq.into_iter().map(DOMString::from_string).collect(),
            ),
        });
        Some((
            IDBObjectStoreParameters {
                autoIncrement: object_store.has_key_generator,
                keyPath: key_path,
            },
            object_store.indexes,
        ))
    }
}

impl IDBTransactionMethods<crate::DomTypeHolder> for IDBTransaction {
    /// <https://www.w3.org/TR/IndexedDB-2/#dom-idbtransaction-db>
    fn Db(&self) -> DomRoot<IDBDatabase> {
        DomRoot::from_ref(&*self.db)
    }

    /// <https://www.w3.org/TR/IndexedDB-2/#dom-idbtransaction-objectstore>
    fn ObjectStore(&self, name: DOMString, can_gc: CanGc) -> Fallible<DomRoot<IDBObjectStore>> {
        // Step 1: If transaction has finished, throw an "InvalidStateError" DOMException.
        if self.finished.get() {
            return Err(Error::InvalidState(None));
        }

        // Step 2: Check that the object store exists
        if !self.object_store_names.Contains(name.clone()) {
            return Err(Error::NotFound(None));
        }

        // Step 3: Each call to this method on the same
        // IDBTransaction instance with the same name
        // returns the same IDBObjectStore instance.
        if let Some(store) = self.store_handles.borrow().get(&*name.str()) {
            return Ok(DomRoot::from_ref(store));
        }

        let parameters = self.object_store_parameters(&name);
        let store = IDBObjectStore::new(
            &self.global(),
            self.db.get_name(),
            name.clone(),
            parameters.as_ref().map(|(params, _)| params),
            can_gc,
            self,
        );
        if let Some(indexes) = parameters.map(|(_, indexes)| indexes) {
            for index in indexes {
                store.add_index(
                    DOMString::from_string(index.name),
                    &IDBIndexParameters {
                        multiEntry: index.multi_entry,
                        unique: index.unique,
                    },
                    index.key_path.into(),
                    can_gc,
                );
            }
        }
        self.store_handles
            .borrow_mut()
            .insert(name.to_string(), Dom::from_ref(&*store));
        Ok(store)
    }

    /// <https://www.w3.org/TR/IndexedDB-2/#commit-transaction>
    fn Commit(&self) -> Fallible<()> {
        // Step 1
        if self.finished.get() {
            return Err(Error::InvalidState(None));
        }

        // https://w3c.github.io/IndexedDB/#transaction-lifecycle
        // “An explicit call to commit() will initiate a commit without waiting for request results
        //  to be handled by script.”
        //
        // Automatic commit additionally requires:
        // The implementation must attempt to commit an inactive transaction
        // when all requests placed against the transaction have completed and
        // their returned results handled, no new requests have been placed against the transaction,
        // and the transaction has not been aborted
        self.set_active_flag(false);
        self.force_commit();

        Ok(())
    }

    /// <https://www.w3.org/TR/IndexedDB-2/#dom-idbtransaction-abort>
    fn Abort(&self) -> Fallible<()> {
        // FIXME:(rasviitanen)
        // This only sets the flags, and does not abort the transaction
        // see https://www.w3.org/TR/IndexedDB-2/#abort-a-transaction
        if self.finished.get() {
            return Err(Error::InvalidState(None));
        }
        self.active.set(false);
        self.initiate_abort(Error::Abort(None), CanGc::note());
        self.request_backend_abort();

        Ok(())
    }

    /// <https://www.w3.org/TR/IndexedDB-2/#dom-idbtransaction-objectstorenames>
    fn ObjectStoreNames(&self) -> DomRoot<DOMStringList> {
        self.object_store_names.as_rooted()
    }

    /// <https://www.w3.org/TR/IndexedDB-2/#dom-idbtransaction-mode>
    fn Mode(&self) -> IDBTransactionMode {
        self.mode
    }

    // https://www.w3.org/TR/IndexedDB-2/#dom-idbtransaction-mode
    // fn Durability(&self) -> IDBTransactionDurability {
    //     // FIXME:(arihant2math) Durability is not implemented at all
    //     unimplemented!();
    // }

    /// <https://www.w3.org/TR/IndexedDB-2/#dom-idbtransaction-error>
    fn GetError(&self) -> Option<DomRoot<DOMException>> {
        self.error.get()
    }

    // https://www.w3.org/TR/IndexedDB-2/#dom-idbtransaction-onabort
    event_handler!(abort, GetOnabort, SetOnabort);

    // https://www.w3.org/TR/IndexedDB-2/#dom-idbtransaction-oncomplete
    event_handler!(complete, GetOncomplete, SetOncomplete);

    // https://www.w3.org/TR/IndexedDB-2/#dom-idbtransaction-onerror
    event_handler!(error, GetOnerror, SetOnerror);
}
