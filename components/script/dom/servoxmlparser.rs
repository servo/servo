/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::js::{JS, Root};
use dom::bindings::reflector::Reflector;
use dom::bindings::trace::JSTraceable;
use dom::document::Document;
use dom::node::Node;
use dom::text::Text;
use url::Url;
use util::str::DOMString;
use xml5ever::tree_builder::{NodeOrText, TreeSink};

#[must_root]
#[derive(JSTraceable, HeapSizeOf)]
pub struct Sink {
    pub base_url: Option<Url>,
    pub document: JS<Document>,
}

impl Sink {
    #[allow(unrooted_must_root)] // method is only run at parse time
    pub fn get_or_create(&self, child: NodeOrText<JS<Node>>) -> Root<Node> {
        match child {
            NodeOrText::AppendNode(n) => Root::from_ref(&*n),
            NodeOrText::AppendText(t) => {
                let s: String = t.into();
                let text = Text::new(DOMString::from(s), &self.document);
                Root::upcast(text)
            }
        }
    }
}
#[must_root]
#[dom_struct]
pub struct ServoXMLParser {
    reflector_: Reflector,
}

impl ServoXMLParser {
    pub fn new() {
    }
}
