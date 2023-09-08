/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::rust::HandleObject;
use xml5ever::serialize::{serialize, SerializeOpts, TraversalScope};

use crate::dom::bindings::codegen::Bindings::XMLSerializerBinding::XMLSerializerMethods;
use crate::dom::bindings::error::{Error, Fallible};
use crate::dom::bindings::reflector::{reflect_dom_object_with_proto, Reflector};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::DOMString;
use crate::dom::node::Node;
use crate::dom::window::Window;

#[dom_struct]
pub struct XMLSerializer {
    reflector_: Reflector,
    window: Dom<Window>,
}

impl XMLSerializer {
    fn new_inherited(window: &Window) -> XMLSerializer {
        XMLSerializer {
            reflector_: Reflector::new(),
            window: Dom::from_ref(window),
        }
    }

    pub fn new(window: &Window, proto: Option<HandleObject>) -> DomRoot<XMLSerializer> {
        reflect_dom_object_with_proto(
            Box::new(XMLSerializer::new_inherited(window)),
            window,
            proto,
        )
    }

    #[allow(non_snake_case)]
    pub fn Constructor(
        window: &Window,
        proto: Option<HandleObject>,
    ) -> Fallible<DomRoot<XMLSerializer>> {
        Ok(XMLSerializer::new(window, proto))
    }
}

impl XMLSerializerMethods for XMLSerializer {
    // https://w3c.github.io/DOM-Parsing/#the-xmlserializer-interface
    fn SerializeToString(&self, root: &Node) -> Fallible<DOMString> {
        let mut writer = vec![];
        match serialize(
            &mut writer,
            &root,
            SerializeOpts {
                traversal_scope: TraversalScope::IncludeNode,
            },
        ) {
            Ok(_) => Ok(DOMString::from(String::from_utf8(writer).unwrap())),
            Err(_) => Err(Error::Type(String::from(
                "root must be a Node or an Attr object",
            ))),
        }
    }
}
