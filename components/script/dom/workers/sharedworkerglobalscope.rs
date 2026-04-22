/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;

use crate::dom::bindings::codegen::Bindings::SharedWorkerGlobalScopeBinding::SharedWorkerGlobalScopeMethods;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::str::DOMString;
use crate::dom::workerglobalscope::WorkerGlobalScope;

// https://html.spec.whatwg.org/multipage/#the-sharedworkerglobalscope-interface
#[dom_struct]
pub(crate) struct SharedWorkerGlobalScope {
    workerglobalscope: WorkerGlobalScope,
}

impl SharedWorkerGlobalScopeMethods<crate::DomTypeHolder> for SharedWorkerGlobalScope {
    /// <https://html.spec.whatwg.org/multipage/#dom-sharedworkerglobalscope-name>
    fn Name(&self) -> DOMString {
        // The name getter steps are to return this's name.
        // Its value represents the name that can be used to obtain a reference to the worker using the SharedWorker constructor.
        self.workerglobalscope.worker_name()
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-sharedworkerglobalscope-close>
    fn Close(&self) {
        // The close() method steps are to close a worker given this.
        self.upcast::<WorkerGlobalScope>().close()
    }

    // <https://html.spec.whatwg.org/multipage/#handler-sharedworkerglobalscope-onconnect>
    event_handler!(connect, GetOnconnect, SetOnconnect);
}
