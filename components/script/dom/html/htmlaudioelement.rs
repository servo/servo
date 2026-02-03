/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::rc::Rc;

use dom_struct::dom_struct;
use html5ever::{LocalName, Prefix, QualName, local_name, ns};
use js::rust::HandleObject;
use style::attr::AttrValue;

use crate::dom::bindings::codegen::Bindings::HTMLAudioElementBinding::HTMLAudioElementMethods;
use crate::dom::bindings::codegen::Bindings::WindowBinding::WindowMethods;
use crate::dom::bindings::error::Fallible;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::document::Document;
use crate::dom::element::{CustomElementCreationMode, Element, ElementCreator};
use crate::dom::html::htmlmediaelement::HTMLMediaElement;
use crate::dom::node::Node;
use crate::dom::window::Window;
use crate::script_runtime::CanGc;

#[dom_struct]
pub(crate) struct HTMLAudioElement {
    htmlmediaelement: HTMLMediaElement,
}

impl HTMLAudioElement {
    fn new_inherited(
        local_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
    ) -> HTMLAudioElement {
        HTMLAudioElement {
            htmlmediaelement: HTMLMediaElement::new_inherited(local_name, prefix, document),
        }
    }

    pub(crate) fn new(
        local_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
        proto: Option<HandleObject>,
        can_gc: CanGc,
    ) -> DomRoot<HTMLAudioElement> {
        Node::reflect_weak_referenceable_node_with_proto(
            Rc::new(HTMLAudioElement::new_inherited(
                local_name, prefix, document,
            )),
            document,
            proto,
            can_gc,
        )
    }
}

impl HTMLAudioElementMethods<crate::DomTypeHolder> for HTMLAudioElement {
    /// <https://html.spec.whatwg.org/multipage/#dom-audio>
    fn Audio(
        window: &Window,
        proto: Option<HandleObject>,
        can_gc: CanGc,
        src: Option<DOMString>,
    ) -> Fallible<DomRoot<HTMLAudioElement>> {
        // Step 1. Let document be the current global object's associated Document.
        let document = window.Document();

        // Step 2. Let audio be the result of creating an element given document, "audio", and the
        // HTML namespace.
        let audio = Element::create(
            QualName::new(None, ns!(html), local_name!("audio")),
            None,
            &document,
            ElementCreator::ScriptCreated,
            CustomElementCreationMode::Synchronous,
            proto,
            can_gc,
        );

        // Step 3. Set an attribute value for audio using "preload" and "auto".
        audio.set_attribute(
            &local_name!("preload"),
            AttrValue::String("auto".to_owned()),
            can_gc,
        );

        // Step 4. If src is given, then set an attribute value for audio using "src" and src. (This
        // will cause the user agent to invoke the object's resource selection algorithm before
        // returning).
        if let Some(s) = src {
            audio.set_attribute(&local_name!("src"), AttrValue::String(s.into()), can_gc);
        }

        // Step 5. Return audio.
        Ok(DomRoot::downcast::<HTMLAudioElement>(audio).unwrap())
    }
}
