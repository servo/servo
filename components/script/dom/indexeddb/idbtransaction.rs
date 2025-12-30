/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;
use std::collections::HashMap;

use base::generic_channel::{GenericSend, GenericSender};
use base::id::ScriptEventLoopId;
use dom_struct::dom_struct;
use profile_traits::generic_channel::channel;
use script_bindings::codegen::GenericUnionTypes::StringOrStringSequence;
use storage_traits::indexeddb::{IndexedDBThreadMsg, KeyPath, SyncOperation};
use stylo_atoms::Atom;

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::DOMStringListBinding::DOMStringListMethods;
use crate::dom::bindings::codegen::Bindings::IDBDatabaseBinding::IDBObjectStoreParameters;
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
    // https://www.w3.org/TR/IndexedDB-2/#transaction-commit
    commit_requested: Cell<bool>,
    // https://www.w3.org/TR/IndexedDB-2/#transaction-commit
    committing: Cell<bool>,
    // Tracks how many IDBRequest instances are still pending for this
    // transaction. The value is incremented when a request is added to the
    // transaction’s request list and decremented once the request has
    // finished.
    pending_request_count: Cell<usize>,
    aborted: Cell<bool>,
    upgrade_old_version: Cell<Option<u64>>,
    // “These steps are invoked by [HTML]… ensure that transactions created by a script call to
    // transaction() are deactivated once the task that invoked the script has completed. The steps
    // are run at most once for each transaction.”
    // https://w3c.github.io/IndexedDB/#cleanup-indexed-database-transactions
    #[no_trace]
    cleanup_event_loop: DomRefCell<Option<ScriptEventLoopId>>,
    cleanup_done: Cell<bool>,
    cleanup_unregistered: Cell<bool>,

    // An unique identifier, used to commit and revert this transaction
    // FIXME:(rasviitanen) Replace this with a channel
    serial_number: u64,
}

impl IDBTransaction {
    fn new_inherited(
        connection: &IDBDatabase,
        mode: IDBTransactionMode,
        scope: &DOMStringList,
        serial_number: u64,
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
            commit_requested: Cell::new(false),
            committing: Cell::new(false),
            pending_request_count: Cell::new(0),
            aborted: Cell::new(false),
            upgrade_old_version: Cell::new(None),
            cleanup_event_loop: DomRefCell::new(None),
            cleanup_done: Cell::new(false),
            cleanup_unregistered: Cell::new(false),
            serial_number,
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
        can_gc: CanGc,
    ) -> DomRoot<IDBTransaction> {
        reflect_dom_object(
            Box::new(IDBTransaction::new_inherited(
                connection,
                mode,
                scope,
                transaction_id,
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
        self.active.set(status);
        if !status {
            self.maybe_finish_successfully();
        }
    }

    pub fn is_active(&self) -> bool {
        self.active.get()
    }

    pub(crate) fn is_aborted(&self) -> bool {
        self.aborted.get()
    }

    pub(crate) fn is_finished(&self) -> bool {
        self.finished.get()
    }

    pub(crate) fn cleanup_event_loop(&self) -> Option<ScriptEventLoopId> {
        *self.cleanup_event_loop.borrow()
    }

    pub(crate) fn is_cleanup_done(&self) -> bool {
        self.cleanup_done.get()
    }

    pub(crate) fn set_cleanup_event_loop(&self, event_loop_id: ScriptEventLoopId) {
        if self.cleanup_done.get() || self.cleanup_unregistered.get() {
            return;
        }
        *self.cleanup_event_loop.borrow_mut() = Some(event_loop_id);
        self.global().register_indexeddb_transaction(self);
    }

    pub(crate) fn mark_cleanup_ran(&self) {
        self.cleanup_done.set(true);
    }

    pub(crate) fn assert_active_for_request(&self) -> Fallible<()> {
        // https://w3c.github.io/IndexedDB/#transaction-lifecycle
        // Once a transaction has committed or aborted, it enters this state.
        // No requests can be made against the transaction when it is in this state.
        // The implementation must allow requests to be placed against the transaction whenever it is active.”

        // Inference for implementation: after commit() is called we must prevent further request placement even if script is still running,
        // because the commit has been explicitly requested and the transaction is transitioning toward finished.
        if self.finished.get() ||
            self.aborted.get() ||
            self.commit_requested.get() ||
            self.committing.get() ||
            !self.active.get()
        {
            return Err(Error::TransactionInactive(None));
        }
        Ok(())
    }

    pub(crate) fn deactivate_for_cleanup(&self) {
        // SPEC (cleanup IndexedDB transactions):
        // “Set transaction’s state to inactive.”
        // https://w3c.github.io/IndexedDB/#cleanup-indexed-database-transactions
        self.active.set(false);
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

    pub fn add_request(&self, request: &IDBRequest) {
        self.requests.borrow_mut().push(Dom::from_ref(request));
        self.inc_pending_requests();
    }

    /// Must be called by an `IDBRequest` when it finishes (either success or
    /// error). When the last pending request has completed and the transaction
    /// is no longer active, the `"complete"` event is dispatched and any
    /// associated `IDBOpenDBRequest` `"success"` event is fired afterwards.
    pub fn request_finished(&self) {
        self.dec_pending_requests();
        if self.aborted.get() {
            self.maybe_finish_after_abort();
        } else {
            self.maybe_finish_successfully();
        }
    }

    pub(crate) fn set_upgrade_metadata(&self, old_version: u64) {
        self.upgrade_old_version.set(Some(old_version));
    }

    pub(crate) fn initiate_abort(&self, error: Error, can_gc: CanGc) {
        // https://w3c.github.io/IndexedDB/#transaction-lifecycle
        // A transaction can be aborted at any time before it is finished,
        // even if the transaction isn’t currently active or hasn’t yet started.”
        // Abort can be initiated any time before the transaction finishes.
        // https://w3c.github.io/IndexedDB/#transaction-lifecycle
        if self.finished.get() || self.aborted.get() {
            return;
        }

        if self.error.get().is_none() {
            if let Ok(exception) = create_dom_exception(&self.global(), error, can_gc) {
                self.error.set(Some(&exception));
            }
        }

        self.aborted.set(true);
        let _ = self
            .get_idb_thread()
            .send(IndexedDBThreadMsg::AbortTransaction {
                origin: self.global().origin().immutable().clone(),
                name: self.db.get_name().to_string(),
                transaction: self.serial_number,
            });
        // https://w3c.github.io/IndexedDB/#abort-a-transaction
        // When a transaction is aborted the implementation must undo (roll back)
        // any changes that were made to the database during that transaction.”
        self.queue_abort_pending_requests();
        // https://w3c.github.io/IndexedDB/#abort-a-transaction
        if let Some(old_version) = self.upgrade_old_version.get() {
            // Keep the DOM-side version consistent until backend cleanup runs.
            self.db.set_version_cache(old_version);
        }

        // NOTE: Completion of abort waits for outstanding requests to finish.
        if self.pending_request_count.get() == 0 {
            self.maybe_finish_after_abort();
        }
    }

    fn queue_abort_pending_requests(&self) {
        // https://w3c.github.io/IndexedDB/#transaction-concept
        // A transaction has a request list of pending requests which have been made against the transaction.
        // When a transaction is aborted the implementation must
        // abort all outstanding requests made against the transaction.
        let requests: Vec<Trusted<IDBRequest>> = self
            .requests
            .borrow()
            .iter()
            .map(|request| Trusted::new(&**request))
            .collect();
        let global = self.global();
        global.task_manager().database_access_task_source().queue(
            task!(abort_pending_requests: move || {
                for request in requests {
                    let request = request.root();
                    request.abort_due_to_transaction(CanGc::note());
                }
            }),
        );
    }

    fn inc_pending_requests(&self) {
        // Track outstanding requests so we can detect when the transaction may finish.
        self.pending_request_count
            .set(self.pending_request_count.get() + 1);
    }

    fn dec_pending_requests(&self) {
        let pending = self.pending_request_count.get();
        if pending == 0 {
            return;
        }
        self.pending_request_count.set(pending - 1);
    }

    fn maybe_finish_successfully(&self) {
        // Finish only after all requests complete and the transaction becomes inactive.
        // https://w3c.github.io/IndexedDB/#transaction-lifecycle
        // The implementation must attempt to commit an inactive transaction when all requests
        // placed against the transaction have completed and their returned results handled,
        // no new requests have been placed against the transaction, and the transaction has not been aborted
        if self.active.get() ||
            self.pending_request_count.get() != 0 ||
            self.finished.get() ||
            self.aborted.get() ||
            self.error.get().is_some()
        {
            return;
        }

        self.finished.set(true);
        self.notify_transaction_finished();
        self.clear_cleanup_event_loop_and_unregister();
        // https://w3c.github.io/IndexedDB/#transaction-lifecycle
        // An event with type complete is fired at a transaction that has successfully committed.
        self.dispatch_complete();
    }

    fn maybe_finish_after_abort(&self) {
        // https://w3c.github.io/IndexedDB/#transaction-concept
        // A transaction has a request list of pending requests which have been made against the transaction.
        if self.finished.get() || self.pending_request_count.get() != 0 {
            return;
        }
        self.finished.set(true);
        self.active.set(false);
        self.clear_cleanup_event_loop_and_unregister();
        self.notify_transaction_finished();
        // https://w3c.github.io/IndexedDB/#transaction-lifecycle
        // An event with type abort is fired at a transaction that has aborted.
        self.dispatch_abort();
    }
    fn notify_transaction_finished(&self) {
        let _ = self
            .get_idb_thread()
            .send(IndexedDBThreadMsg::TransactionFinished {
                origin: self.global().origin().immutable().clone(),
                name: self.db.get_name().to_string(),
                transaction: self.serial_number,
            });
    }

    pub(crate) fn clear_cleanup_event_loop_and_unregister(&self) {
        if self.cleanup_unregistered.replace(true) {
            return;
        }
        // Clears the per-transaction cleanup event loop and removes this transaction from the GlobalScope list.
        // This is bookkeeping to avoid keeping finished transactions alive.
        // Note: this is not the spec “cleanup algorithm”; the spec algorithm is invoked by HTML at end of task.
        // https://w3c.github.io/IndexedDB/#cleanup-indexed-database-transactions
        *self.cleanup_event_loop.borrow_mut() = None;
        self.global().unregister_indexeddb_transaction(self);
    }

    fn dispatch_abort(&self) {
        let global = self.global();
        let this = Trusted::new(self);
        global.task_manager().database_access_task_source().queue(
            task!(send_abort_notification: move || {
                let this = this.root();
                let global = this.global();
                let event = Event::new(
                    &global,
                    Atom::from("abort"),
                    EventBubbles::DoesNotBubble,
                    EventCancelable::NotCancelable,
                    CanGc::note()
                );
                event.fire(this.upcast(), CanGc::note());
            }),
        );
    }

    fn dispatch_complete(&self) {
        let global = self.global();
        let this = Trusted::new(self);
        global.task_manager().database_access_task_source().queue(
            task!(send_complete_notification: move || {
                let this = this.root();
                let global = this.global();
                let event = Event::new(
                    &global,
                    Atom::from("complete"),
                    EventBubbles::DoesNotBubble,
                    EventCancelable::NotCancelable,
                    CanGc::note()
                );
                event.fire(this.upcast(), CanGc::note());
            }),
        );
    }

    fn get_idb_thread(&self) -> GenericSender<IndexedDBThreadMsg> {
        self.global().storage_threads().sender()
    }

    fn object_store_parameters(
        &self,
        object_store_name: &DOMString,
    ) -> Option<IDBObjectStoreParameters> {
        let global = self.global();
        let idb_sender = global.storage_threads().sender();
        let (sender, receiver) =
            channel(global.time_profiler_chan().clone()).expect("failed to create channel");

        let origin = global.origin().immutable().clone();
        let db_name = self.db.get_name().to_string();
        let object_store_name = object_store_name.to_string();

        let operation = SyncOperation::HasKeyGenerator(
            sender,
            origin.clone(),
            db_name.clone(),
            object_store_name.clone(),
        );

        let _ = idb_sender.send(IndexedDBThreadMsg::Sync(operation));

        // First unwrap for ipc
        // Second unwrap will never happen unless this db gets manually deleted somehow
        let auto_increment = receiver.recv().ok()?.ok()?;

        let (sender, receiver) = channel(self.global().time_profiler_chan().clone())?;
        let operation = SyncOperation::KeyPath(sender, origin, db_name, object_store_name);

        let _ = idb_sender.send(IndexedDBThreadMsg::Sync(operation));

        // First unwrap for ipc
        // Second unwrap will never happen unless this db gets manually deleted somehow
        let key_path = receiver.recv().unwrap().ok()?;
        let key_path = key_path.map(|key_path| match key_path {
            KeyPath::String(s) => StringOrStringSequence::String(DOMString::from_string(s)),
            KeyPath::Sequence(seq) => StringOrStringSequence::StringSequence(
                seq.into_iter().map(DOMString::from_string).collect(),
            ),
        });
        Some(IDBObjectStoreParameters {
            autoIncrement: auto_increment,
            keyPath: key_path,
        })
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
            parameters.as_ref(),
            can_gc,
            self,
        );
        self.store_handles
            .borrow_mut()
            .insert(name.to_string(), Dom::from_ref(&*store));
        Ok(store)
    }

    /// <https://www.w3.org/TR/IndexedDB-2/#commit-transaction>
    fn Commit(&self) -> Fallible<()> {
        // https://w3c.github.io/IndexedDB/#transaction-lifecycle
        // An explicit call to commit() will initiate a commit without waiting for request results to be handled by script.
        // Step 1
        self.commit_requested.set(true);
        self.committing.set(true);
        let (sender, receiver) = channel(self.global().time_profiler_chan().clone()).unwrap();
        let start_operation = SyncOperation::Commit(
            sender,
            self.global().origin().immutable().clone(),
            self.db.get_name().to_string(),
            self.serial_number,
        );

        self.get_idb_thread()
            .send(IndexedDBThreadMsg::Sync(start_operation))
            .unwrap();

        let result = receiver.recv().unwrap();

        // Step 2
        if let Err(_result) = result {
            // FIXME:(rasviitanen) also support Unknown error
            return Err(Error::QuotaExceeded {
                quota: None,
                requested: None,
            });
        }

        // Step 3
        // FIXME:(rasviitanen) https://www.w3.org/TR/IndexedDB-2/#commit-a-transaction

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
        self.initiate_abort(Error::Abort(None), CanGc::note());

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
