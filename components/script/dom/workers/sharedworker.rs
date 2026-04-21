/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::context::JSContext;
use js::rust::HandleObject;

use crate::dom::bindings::codegen::Bindings::SharedWorkerBinding::SharedWorkerMethods;
use crate::dom::bindings::codegen::UnionTypes::{
    StringOrSharedWorkerOptions, TrustedScriptURLOrUSVString,
};
use crate::dom::bindings::error::{Error, Fallible};
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::reflector::reflect_dom_object_with_proto_and_cx;
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::eventtarget::EventTarget;
use crate::dom::globalscope::GlobalScope;
use crate::dom::messageport::MessagePort;
use crate::dom::trustedtypes::trustedscripturl::TrustedScriptURL;
use crate::dom::window::Window;
use crate::script_runtime::CanGc;

/// <https://html.spec.whatwg.org/multipage/#shared-workers-and-the-sharedworker-interface>
#[dom_struct]
pub(crate) struct SharedWorker {
    eventtarget: EventTarget,
    /// The outside port returned to the creator's global.
    port: Dom<MessagePort>,
}

impl SharedWorker {
    fn new_inherited(port: &MessagePort) -> SharedWorker {
        SharedWorker {
            eventtarget: EventTarget::new_inherited(),
            port: Dom::from_ref(port),
        }
    }

    fn new(
        global: &GlobalScope,
        proto: Option<HandleObject>,
        port: &MessagePort,
        cx: &mut js::context::JSContext,
    ) -> DomRoot<SharedWorker> {
        reflect_dom_object_with_proto_and_cx(
            Box::new(SharedWorker::new_inherited(port)),
            global,
            proto,
            cx,
        )
    }
}

impl SharedWorkerMethods<crate::DomTypeHolder> for SharedWorker {
    /// <https://html.spec.whatwg.org/multipage/#dom-sharedworker>
    fn Constructor(
        cx: &mut JSContext,
        window: &Window,
        proto: Option<HandleObject>,
        script_url: TrustedScriptURLOrUSVString,
        options: StringOrSharedWorkerOptions,
    ) -> Fallible<DomRoot<SharedWorker>> {
        let global = window.upcast::<GlobalScope>();

        // Step 1. Let compliantScriptURL be the result of invoking the get trusted type
        // compliant string algorithm with TrustedScriptURL, this's relevant global object,
        // scriptURL, "SharedWorker constructor", and "script".
        let compliant_script_url = TrustedScriptURL::get_trusted_type_compliant_string(
            cx,
            global,
            script_url,
            "SharedWorker constructor",
        )?;

        // Step 2. If options is a DOMString, set options to a new SharedWorkerOptions
        // dictionary whose name member is set to the value of options and whose other
        // members are set to their default values.
        match options {
            StringOrSharedWorkerOptions::String(name) => {
                // TODO: The name will be used later.
                let _worker_name = name;
            },
            StringOrSharedWorkerOptions::SharedWorkerOptions(_opts) => {
                // TODO: Extract name from opts when implementing the registry phase.
            },
        }

        // Step 3. Let outsideSettings be this's relevant settings object.
        // (outsideSettings is `global` throughout.)

        // Step 4. Let urlRecord be the result of encoding-parsing a URL given
        // compliantScriptURL, relative to outsideSettings.
        // Step 5. If urlRecord is failure, then throw a "SyntaxError" DOMException.
        let Ok(_worker_url) = global.encoding_parse_a_url(&compliant_script_url.str()) else {
            return Err(Error::Syntax(None));
        };

        // Step 6. Let outsidePort be a new MessagePort in outsideSettings's realm.
        let outside_port = MessagePort::new(global, CanGc::from_cx(cx));
        global.track_message_port(&outside_port, None);

        // Step 7. Set this's port to outsidePort.
        // (Stored via SharedWorker::new below.)

        // TODO Step 8. Let callerIsSecureContext be true if outsideSettings is a secure
        // context; otherwise, false.

        // TODO Step 9. Let outsideStorageKey be the result of running obtain a storage key
        // for non-storage purposes given outsideSettings.

        // TODO Step 10. Let worker be this.

        // TODO Step 11

        Ok(SharedWorker::new(global, proto, &outside_port, cx))
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-sharedworker-port>
    fn Port(&self) -> DomRoot<MessagePort> {
        // The port getter steps are to return this's port.
        DomRoot::from_ref(&*self.port)
    }

    // <https://html.spec.whatwg.org/multipage/#handler-abstractworker-onerror>
    event_handler!(error, GetOnerror, SetOnerror);
}
