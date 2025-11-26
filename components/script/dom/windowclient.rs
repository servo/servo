/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::rc::Rc;

use base::id::PipelineId;
use dom_struct::dom_struct;
use script_bindings::codegen::GenericBindings::WindowClientBinding::WindowClientMethods;
use script_bindings::root::DomRoot;
use script_bindings::script_runtime::CanGc;
use script_bindings::str::USVString;

use crate::dom::bindings::reflector::{DomGlobal, reflect_dom_object};
use crate::dom::client::Client;
use crate::dom::globalscope::GlobalScope;
use crate::dom::promise::Promise;

#[dom_struct]
pub(crate) struct WindowClient {
    client: Client,
    #[no_trace]
    pipeline_id: PipelineId,
}

impl WindowClient {
    pub(crate) fn new_inherited(client: Client, pipeline_id: PipelineId) -> WindowClient {
        WindowClient {
            client,
            pipeline_id,
        }
    }

    pub(crate) fn new(
        global: &GlobalScope,
        pipeline_id: PipelineId,
        can_gc: CanGc,
    ) -> DomRoot<WindowClient> {
        reflect_dom_object(
            Box::new(WindowClient::new_inherited(
                Client::new_inherited(global.get_url()),
                pipeline_id,
            )),
            global,
            can_gc,
        )
    }
}

impl WindowClientMethods<crate::DomTypeHolder> for WindowClient {
    /// <https://w3c.github.io/ServiceWorker/#dom-windowclient-focus>
    fn Focus(&self) -> Rc<Promise> {
        // TODO: Implement
        Promise::new(&self.global(), CanGc::note())
    }

    /// <https://w3c.github.io/ServiceWorker/#dom-windowclient-navigate>
    fn Navigate(&self, _url: USVString) -> Rc<Promise> {
        // TODO: Implement
        Promise::new(&self.global(), CanGc::note())
    }
}
