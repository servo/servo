/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::context::JSContext;
use js::rust::HandleObject;
use script_bindings::reflector::{Reflector, reflect_dom_object_with_proto_and_cx};
use xml5ever::serialize::{SerializeOpts, TraversalScope, serialize};

use crate::dom::bindings::codegen::Bindings::XMLSerializerBinding::XMLSerializerMethods;
use crate::dom::bindings::error::{Error, Fallible};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::DOMString;
use crate::dom::node::Node;
use crate::dom::servoparser::html::HtmlSerialize;
use crate::dom::window::Window;

#[dom_struct]
pub(crate) struct XMLSerializer {
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

    pub(crate) fn new(
        cx: &mut JSContext,
        window: &Window,
        proto: Option<HandleObject>,
    ) -> DomRoot<XMLSerializer> {
        reflect_dom_object_with_proto_and_cx(
            Box::new(XMLSerializer::new_inherited(window)),
            window,
            proto,
            cx,
        )
    }
}

impl XMLSerializerMethods<crate::DomTypeHolder> for XMLSerializer {
    /// <https://w3c.github.io/DOM-Parsing/#dom-xmlserializer>
    fn Constructor(
        cx: &mut JSContext,
        window: &Window,
        proto: Option<HandleObject>,
    ) -> Fallible<DomRoot<XMLSerializer>> {
        Ok(XMLSerializer::new(cx, window, proto))
    }

    /// <https://w3c.github.io/DOM-Parsing/#the-xmlserializer-interface>
    fn SerializeToString(&self, root: &Node) -> Fallible<DOMString> {
        let mut writer = vec![];
        match serialize(
            &mut writer,
            &HtmlSerialize::new(root),
            SerializeOpts {
                traversal_scope: TraversalScope::IncludeNode,
            },
        ) {
            Ok(_) => Ok(DOMString::from(String::from_utf8(writer).unwrap())),
            Err(_) => Err(Error::Type(
                c"root must be a Node or an Attr object".to_owned(),
            )),
        }
    }
}
