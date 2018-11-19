/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::codegen::Bindings::HTMLAudioElementBinding;
use crate::dom::bindings::codegen::Bindings::HTMLMediaElementBinding::HTMLMediaElementBinding::HTMLMediaElementMethods;
use crate::dom::bindings::codegen::Bindings::WindowBinding::WindowMethods;
use crate::dom::bindings::error::Fallible;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::document::Document;
use crate::dom::htmlmediaelement::HTMLMediaElement;
use crate::dom::node::Node;
use crate::dom::window::Window;
use dom_struct::dom_struct;
use html5ever::{LocalName, Prefix};

#[dom_struct]
pub struct HTMLAudioElement {
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

    #[allow(unrooted_must_root)]
    pub fn new(
        local_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
    ) -> DomRoot<HTMLAudioElement> {
        Node::reflect_node(
            Box::new(HTMLAudioElement::new_inherited(
                local_name, prefix, document,
            )),
            document,
            HTMLAudioElementBinding::Wrap,
        )
    }

    pub fn Audio(
        window: &Window,
        src: Option<DOMString>,
    ) -> Fallible<DomRoot<HTMLAudioElement>> {
        let document = window.Document();
        let audio = HTMLAudioElement::new(local_name!("audio"), None, &document);

        audio.upcast::<HTMLMediaElement>().SetPreload(DOMString::from("auto"));
        if let Some(s) = src {
            audio.upcast::<HTMLMediaElement>().SetSrc(s);
        }

        Ok(audio)
    }
}
