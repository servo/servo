/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::rust::HandleObject;
use stylo_atoms::Atom;

use super::bindings::reflector::reflect_dom_object_with_proto;
use crate::dom::bindings::codegen::Bindings::EventBinding::Event_Binding::EventMethods;
use crate::dom::bindings::codegen::Bindings::SecurityPolicyViolationEventBinding::{
    SecurityPolicyViolationEventDisposition, SecurityPolicyViolationEventInit,
    SecurityPolicyViolationEventMethods,
};
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::{DOMString, USVString};
use crate::dom::event::{Event, EventBubbles, EventCancelable, EventComposed};
use crate::dom::globalscope::GlobalScope;
use crate::script_runtime::CanGc;

// https://w3c.github.io/webappsec-csp/#securitypolicyviolationevent
#[dom_struct]
pub(crate) struct SecurityPolicyViolationEvent {
    event: Event,
    document_uri: USVString,
    referrer: USVString,
    blocked_uri: USVString,
    effective_directive: DOMString,
    violated_directive: DOMString,
    original_policy: DOMString,
    source_file: USVString,
    sample: DOMString,
    disposition: SecurityPolicyViolationEventDisposition,
    status_code: u16,
    line_number: u32,
    column_number: u32,
}

impl SecurityPolicyViolationEvent {
    fn new_inherited(init: &SecurityPolicyViolationEventInit) -> SecurityPolicyViolationEvent {
        SecurityPolicyViolationEvent {
            event: Event::new_inherited(),
            document_uri: init.documentURI.clone(),
            referrer: init.referrer.clone(),
            blocked_uri: init.blockedURI.clone(),
            effective_directive: init.effectiveDirective.clone(),
            violated_directive: init.violatedDirective.clone(),
            original_policy: init.originalPolicy.clone(),
            source_file: init.sourceFile.clone(),
            sample: init.sample.clone(),
            disposition: init.disposition,
            status_code: init.statusCode,
            line_number: init.lineNumber,
            column_number: init.columnNumber,
        }
    }

    pub(crate) fn new_initialized(
        global: &GlobalScope,
        init: &SecurityPolicyViolationEventInit,
        proto: Option<HandleObject>,
        can_gc: CanGc,
    ) -> DomRoot<SecurityPolicyViolationEvent> {
        reflect_dom_object_with_proto(
            Box::new(SecurityPolicyViolationEvent::new_inherited(init)),
            global,
            proto,
            can_gc,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn new_with_proto(
        global: &GlobalScope,
        proto: Option<HandleObject>,
        type_: Atom,
        bubbles: EventBubbles,
        cancelable: EventCancelable,
        composed: EventComposed,
        init: &SecurityPolicyViolationEventInit,
        can_gc: CanGc,
    ) -> DomRoot<Self> {
        let ev = SecurityPolicyViolationEvent::new_initialized(global, init, proto, can_gc);
        {
            let event = ev.upcast::<Event>();
            event.init_event(type_, bool::from(bubbles), bool::from(cancelable));
            event.set_composed(bool::from(composed));
        }
        ev
    }

    pub(crate) fn new(
        global: &GlobalScope,
        type_: Atom,
        bubbles: EventBubbles,
        cancelable: EventCancelable,
        composed: EventComposed,
        init: &SecurityPolicyViolationEventInit,
        can_gc: CanGc,
    ) -> DomRoot<Self> {
        Self::new_with_proto(
            global, None, type_, bubbles, cancelable, composed, init, can_gc,
        )
    }
}

#[allow(non_snake_case)]
impl SecurityPolicyViolationEventMethods<crate::DomTypeHolder> for SecurityPolicyViolationEvent {
    /// <https://w3c.github.io/webappsec-csp/#dom-securitypolicyviolationevent-securitypolicyviolationevent>
    fn Constructor(
        global: &GlobalScope,
        proto: Option<HandleObject>,
        can_gc: CanGc,
        type_: DOMString,
        init: &SecurityPolicyViolationEventInit,
    ) -> DomRoot<Self> {
        SecurityPolicyViolationEvent::new_with_proto(
            global,
            proto,
            Atom::from(type_),
            EventBubbles::from(init.parent.bubbles),
            EventCancelable::from(init.parent.cancelable),
            EventComposed::from(init.parent.composed),
            init,
            can_gc,
        )
    }

    /// <https://w3c.github.io/webappsec-csp/#dom-securitypolicyviolationevent-documenturi>
    fn DocumentURI(&self) -> USVString {
        self.document_uri.clone()
    }

    /// <https://w3c.github.io/webappsec-csp/#dom-securitypolicyviolationevent-referrer>
    fn Referrer(&self) -> USVString {
        self.referrer.clone()
    }

    /// <https://w3c.github.io/webappsec-csp/#dom-securitypolicyviolationevent-blockeduri>
    fn BlockedURI(&self) -> USVString {
        self.blocked_uri.clone()
    }

    /// <https://w3c.github.io/webappsec-csp/#dom-securitypolicyviolationevent-effectivedirective>
    fn EffectiveDirective(&self) -> DOMString {
        self.effective_directive.clone()
    }

    /// <https://w3c.github.io/webappsec-csp/#dom-securitypolicyviolationevent-violateddirective>
    fn ViolatedDirective(&self) -> DOMString {
        self.violated_directive.clone()
    }

    /// <https://w3c.github.io/webappsec-csp/#dom-securitypolicyviolationevent-originalpolicy>
    fn OriginalPolicy(&self) -> DOMString {
        self.original_policy.clone()
    }

    /// <https://w3c.github.io/webappsec-csp/#dom-securitypolicyviolationevent-sourcefile>
    fn SourceFile(&self) -> USVString {
        self.source_file.clone()
    }

    /// <https://w3c.github.io/webappsec-csp/#dom-securitypolicyviolationevent-sample>
    fn Sample(&self) -> DOMString {
        self.sample.clone()
    }

    /// <https://w3c.github.io/webappsec-csp/#dom-securitypolicyviolationevent-disposition>
    fn Disposition(&self) -> SecurityPolicyViolationEventDisposition {
        self.disposition
    }

    /// <https://w3c.github.io/webappsec-csp/#dom-securitypolicyviolationevent-statuscode>
    fn StatusCode(&self) -> u16 {
        self.status_code
    }

    /// <https://w3c.github.io/webappsec-csp/#dom-securitypolicyviolationevent-linenumber>
    fn LineNumber(&self) -> u32 {
        self.line_number
    }

    /// <https://w3c.github.io/webappsec-csp/#dom-securitypolicyviolationevent-columnnumber>
    fn ColumnNumber(&self) -> u32 {
        self.column_number
    }

    /// <https://dom.spec.whatwg.org/#dom-event-istrusted>
    fn IsTrusted(&self) -> bool {
        self.event.IsTrusted()
    }
}
