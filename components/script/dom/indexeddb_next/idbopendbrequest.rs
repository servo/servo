/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;

use crate::dom::bindings::codegen::Bindings::IDBOpenDBRequestBinding::IDBOpenDBRequestMethods;
use crate::dom::bindings::reflector::reflect_dom_object;
use crate::dom::bindings::root::DomRoot;
use crate::dom::globalscope::GlobalScope;
use crate::dom::indexeddb_next::idbrequest::IDBRequest;
use crate::script_runtime::CanGc;

/// An "object" implementing the spec’s IDBOpenDBRequest interface:
/// <https://w3c.github.io/IndexedDB/#idbopendbrequest>.
///
/// The IDBOpenDBRequest interface extends IDBRequest and allows listening for
/// additional events.
///
/// The IDBOpenDBRequest struct has a remote counterpart in the backend, which
/// performs some of the steps defined by the corresponding spec algorithms.
#[dom_struct]
pub struct IDBOpenDBRequest {
    request: IDBRequest,
}

impl IDBOpenDBRequest {
    pub fn _new_inherited() -> IDBOpenDBRequest {
        IDBOpenDBRequest {
            request: IDBRequest::_new_inherited(),
        }
    }

    pub fn _new(global: &GlobalScope, can_gc: CanGc) -> DomRoot<IDBOpenDBRequest> {
        reflect_dom_object(Box::new(IDBOpenDBRequest::_new_inherited()), global, can_gc)
    }
}

#[expect(unused_doc_comments)]
impl IDBOpenDBRequestMethods<crate::DomTypeHolder> for IDBOpenDBRequest {
    /// <https://w3c.github.io/IndexedDB/#dom-idbopendbrequest-onblocked>
    event_handler!(blocked, GetOnblocked, SetOnblocked);

    /// <https://w3c.github.io/IndexedDB/#dom-idbopendbrequest-onupgradeneeded>
    event_handler!(upgradeneeded, GetOnupgradeneeded, SetOnupgradeneeded);
}
