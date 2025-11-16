/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;

use dom_struct::dom_struct;

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::IDBDatabaseBinding::{
    IDBDatabaseMethods, IDBObjectStoreParameters, IDBTransactionOptions,
};
use crate::dom::bindings::codegen::Bindings::IDBTransactionBinding::IDBTransactionMode;
use crate::dom::bindings::codegen::UnionTypes::StringOrStringSequence;
use crate::dom::bindings::error::{Error, Fallible};
use crate::dom::bindings::reflector::{DomGlobal, reflect_dom_object};
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::domstringlist::DOMStringList;
use crate::dom::eventtarget::EventTarget;
use crate::dom::globalscope::GlobalScope;
use crate::dom::indexeddb_next::idbobjectstore::IDBObjectStore;
use crate::dom::indexeddb_next::idbtransaction::IDBTransaction;
use crate::script_runtime::CanGc;

/// An "object" implementing the spec’s IDBDatabase interface:
/// <https://w3c.github.io/IndexedDB/#database-interface>.
///
/// The IDBDatabase interface represents a connection to a database:
/// <https://w3c.github.io/IndexedDB/#database-connection>.
///
/// A connection can be used to manipulate the objects of the associated
/// database.
///
/// The IDBDatabase struct has a remote counterpart in the backend, which
/// performs some of the steps defined by the corresponding spec algorithms.
#[dom_struct]
pub struct IDBDatabase {
    eventtarget: EventTarget,

    /// <https://w3c.github.io/IndexedDB/#database-name>
    name: DOMString,

    /// <https://w3c.github.io/IndexedDB/#connection-version>
    version: Cell<u64>,

    object_store_names: DomRefCell<Vec<DOMString>>,
}

impl IDBDatabase {
    pub fn _new_inherited(name: DOMString, version: u64) -> IDBDatabase {
        IDBDatabase {
            eventtarget: EventTarget::new_inherited(),
            name,
            version: Cell::new(version),
            object_store_names: Default::default(),
        }
    }

    pub fn _new(
        global: &GlobalScope,
        name: DOMString,
        version: u64,
        can_gc: CanGc,
    ) -> DomRoot<IDBDatabase> {
        reflect_dom_object(
            Box::new(IDBDatabase::_new_inherited(name, version)),
            global,
            can_gc,
        )
    }
}

#[expect(unused_doc_comments)]
impl IDBDatabaseMethods<crate::DomTypeHolder> for IDBDatabase {
    /// <https://w3c.github.io/IndexedDB/#dom-idbdatabase-name>
    fn Name(&self) -> DOMString {
        self.name.clone()
    }

    /// <https://w3c.github.io/IndexedDB/#dom-idbdatabase-version>
    fn Version(&self) -> u64 {
        self.version.get()
    }

    /// <https://w3c.github.io/IndexedDB/#dom-idbdatabase-objectstorenames>
    fn ObjectStoreNames(&self) -> DomRoot<DOMStringList> {
        DOMStringList::new(
            &self.global(),
            self.object_store_names.borrow().clone(),
            CanGc::note(),
        )
    }

    /// <https://w3c.github.io/IndexedDB/#dom-idbdatabase-transaction>
    fn Transaction(
        &self,
        _store_names: StringOrStringSequence,
        _mode: IDBTransactionMode,
        _options: &IDBTransactionOptions,
    ) -> Fallible<DomRoot<IDBTransaction>> {
        Err(Error::NotSupported)
    }

    /// <https://w3c.github.io/IndexedDB/#dom-idbdatabase-close>
    fn Close(&self) {}

    /// <https://w3c.github.io/IndexedDB/#dom-idbdatabase-createobjectstore>
    fn CreateObjectStore(
        &self,
        _name: DOMString,
        _options: &IDBObjectStoreParameters,
    ) -> Fallible<DomRoot<IDBObjectStore>> {
        Err(Error::NotSupported)
    }

    /// <https://w3c.github.io/IndexedDB/#dom-idbdatabase-deleteobjectstore>
    fn DeleteObjectStore(&self, _name: DOMString) -> Fallible<()> {
        Err(Error::NotSupported)
    }

    /// <https://w3c.github.io/IndexedDB/#dom-idbdatabase-onabort>
    event_handler!(abort, GetOnabort, SetOnabort);

    /// <https://w3c.github.io/IndexedDB/#dom-idbdatabase-onclose>
    event_handler!(close, GetOnclose, SetOnclose);

    /// <https://w3c.github.io/IndexedDB/#dom-idbdatabase-onerror>
    event_handler!(error, GetOnerror, SetOnerror);

    /// <https://w3c.github.io/IndexedDB/#dom-idbdatabase-onversionchange>>
    event_handler!(versionchange, GetOnversionchange, SetOnversionchange);
}
