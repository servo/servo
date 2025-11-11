/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::IDBTransactionBinding::{
    IDBTransactionMethods, IDBTransactionMode,
};
use crate::dom::bindings::error::{Error, Fallible};
use crate::dom::bindings::reflector::{DomGlobal, reflect_dom_object};
use crate::dom::bindings::root::{Dom, DomRoot, MutNullableDom};
use crate::dom::bindings::str::DOMString;
use crate::dom::domexception::DOMException;
use crate::dom::domstringlist::DOMStringList;
use crate::dom::eventtarget::EventTarget;
use crate::dom::globalscope::GlobalScope;
use crate::dom::indexeddb_next::idbdatabase::IDBDatabase;
use crate::dom::indexeddb_next::idbobjectstore::IDBObjectStore;
use crate::script_runtime::CanGc;

/// An "object" implementing the spec’s IDBTransaction interface:
/// <https://w3c.github.io/IndexedDB/#transaction>.
///
/// The IDBTransaction interface represents a transaction:
/// <https://w3c.github.io/IndexedDB/#transaction-construct>.
///
/// A transaction is used to interact with the data stored in the database
/// of the associated connection.
///
/// The IDBTransaction struct has a remote counterpart in the backend, which
/// performs some of the steps defined by the corresponding spec algorithms.
#[dom_struct]
pub struct IDBTransaction {
    eventtarget: EventTarget,

    /// <https://w3c.github.io/IndexedDB/#transaction-mode>
    mode: IDBTransactionMode,
    /// <https://w3c.github.io/IndexedDB/#transaction-connection>
    connection: Dom<IDBDatabase>,
    /// <https://w3c.github.io/IndexedDB/#transaction-error>
    error: MutNullableDom<DOMException>,

    object_store_names: DomRefCell<Vec<DOMString>>,
}

impl IDBTransaction {
    fn _new_inherited(mode: IDBTransactionMode, connection: &IDBDatabase) -> IDBTransaction {
        IDBTransaction {
            eventtarget: EventTarget::new_inherited(),
            mode,
            connection: Dom::from_ref(connection),
            error: Default::default(),
            object_store_names: Default::default(),
        }
    }

    pub fn _new(
        global: &GlobalScope,
        mode: IDBTransactionMode,
        connection: &IDBDatabase,
        can_gc: CanGc,
    ) -> DomRoot<IDBTransaction> {
        reflect_dom_object(
            Box::new(IDBTransaction::_new_inherited(mode, connection)),
            global,
            can_gc,
        )
    }
}

#[expect(unused_doc_comments)]
impl IDBTransactionMethods<crate::DomTypeHolder> for IDBTransaction {
    /// <https://w3c.github.io/IndexedDB/#dom-idbtransaction-objectstorenames>
    fn ObjectStoreNames(&self) -> DomRoot<DOMStringList> {
        DOMStringList::new(
            &self.global(),
            self.object_store_names.borrow().clone(),
            CanGc::note(),
        )
    }

    /// <https://w3c.github.io/IndexedDB/#dom-idbtransaction-mode>
    fn Mode(&self) -> IDBTransactionMode {
        self.mode
    }

    // /// <https://w3c.github.io/IndexedDB/#dom-idbtransaction-durability>
    // fn Durability(&self) -> IDBTransactionDurability {
    //     // FIXME:(arihant2math) Durability is not implemented at all
    //     unimplemented!();
    // }

    /// <https://w3c.github.io/IndexedDB/#dom-idbtransaction-db>
    fn Db(&self) -> DomRoot<IDBDatabase> {
        DomRoot::from_ref(&*self.connection)
    }

    /// <https://w3c.github.io/IndexedDB/#dom-idbtransaction-error>
    fn GetError(&self) -> Option<DomRoot<DOMException>> {
        self.error.get()
    }

    /// <https://w3c.github.io/IndexedDB/#dom-idbtransaction-objectstore>
    fn ObjectStore(&self, _name: DOMString, _can_gc: CanGc) -> Fallible<DomRoot<IDBObjectStore>> {
        Err(Error::NotSupported)
    }

    /// <https://w3c.github.io/IndexedDB/#dom-idbtransaction-commit>
    fn Commit(&self) -> Fallible<()> {
        Err(Error::NotSupported)
    }

    /// <https://w3c.github.io/IndexedDB/#dom-idbtransaction-abort>
    fn Abort(&self) -> Fallible<()> {
        Err(Error::NotSupported)
    }

    /// <https://w3c.github.io/IndexedDB/#dom-idbtransaction-onabort>
    event_handler!(abort, GetOnabort, SetOnabort);

    /// <https://w3c.github.io/IndexedDB/#dom-idbtransaction-oncomplete>
    event_handler!(complete, GetOncomplete, SetOncomplete);

    /// <https://w3c.github.io/IndexedDB/#dom-idbtransaction-onerror>
    event_handler!(error, GetOnerror, SetOnerror);
}
