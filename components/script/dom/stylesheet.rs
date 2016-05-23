/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use core::ops::Deref;
use dom::bindings::codegen::Bindings::StyleSheetBinding;
use dom::bindings::codegen::Bindings::StyleSheetBinding::StyleSheetMethods;
use dom::bindings::codegen::UnionTypes;
use dom::bindings::global::GlobalRef;
use dom::bindings::inheritance::Castable;
use dom::bindings::js::{JS, Root};
use dom::bindings::reflector::{Reflector, reflect_dom_object};
use dom::element::Element;
use dom::node::Node;
use dom::processinginstruction::ProcessingInstruction;
use dom::window::Window;
use util::str::DOMString;

#[dom_struct]
pub struct StyleSheet {
    reflector_: Reflector,
    type_: DOMString,
    href: Option<DOMString>,
    title: Option<DOMString>,
    owner: Option<JS<Node>>,
}

impl StyleSheet {
    #[allow(unrooted_must_root)]
    pub fn new_inherited(type_: DOMString,
                         href: Option<DOMString>,
                         title: Option<DOMString>,
                         owner: Option<&Node>) -> StyleSheet {
        StyleSheet {
            reflector_: Reflector::new(),
            type_: type_,
            href: href,
            title: title,
            owner: owner.map(|node: &Node| {
              JS::from_ref(node)
            })
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(window: &Window, type_: DOMString,
               href: Option<DOMString>,
               title: Option<DOMString>,
               owner: Option<&Node>) -> Root<StyleSheet> {
        reflect_dom_object(box StyleSheet::new_inherited(type_, href, title, owner),
                           GlobalRef::Window(window),
                           StyleSheetBinding::Wrap)
    }
}


impl StyleSheetMethods for StyleSheet {
    // https://drafts.csswg.org/cssom/#dom-stylesheet-type
    fn Type_(&self) -> DOMString {
        self.type_.clone()
    }

    // https://drafts.csswg.org/cssom/#dom-stylesheet-href
    fn GetHref(&self) -> Option<DOMString> {
        self.href.clone()
    }

    // https://drafts.csswg.org/cssom/#dom-stylesheet-title
    fn GetTitle(&self) -> Option<DOMString> {
        self.title.clone()
    }

    // https://drafts.csswg.org/cssom/#dom-stylesheet-ownernode
    fn GetOwnerNode(&self) -> Option<UnionTypes::ElementOrProcessingInstruction>{
        self.owner.as_ref().map(|owner: &JS<Node>| {
          if let Some(elem) = Root::downcast::<Element>(Root::from_ref(owner.deref())) {
            UnionTypes::ElementOrProcessingInstruction::Element(elem)
          } else {
            let instr: Option<Root<ProcessingInstruction>> =
                Root::downcast::<ProcessingInstruction>(Root::from_ref(owner.deref()));
            UnionTypes::ElementOrProcessingInstruction::ProcessingInstruction(instr.unwrap())
          }
        })
    }
}
