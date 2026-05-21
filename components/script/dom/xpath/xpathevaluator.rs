/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::rc::Rc;

use dom_struct::dom_struct;
use js::context::JSContext;
use js::rust::HandleObject;
use script_bindings::codegen::GenericBindings::WindowBinding::WindowMethods;
use script_bindings::reflector::{Reflector, reflect_dom_object_with_proto_and_cx};

use crate::dom::bindings::codegen::Bindings::XPathEvaluatorBinding::XPathEvaluatorMethods;
use crate::dom::bindings::codegen::Bindings::XPathNSResolverBinding::XPathNSResolver;
use crate::dom::bindings::error::Fallible;
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::DOMString;
use crate::dom::node::Node;
use crate::dom::window::Window;
use crate::dom::xpathexpression::XPathExpression;
use crate::dom::xpathresult::XPathResult;
use crate::xpath::parse_expression;

#[dom_struct]
pub(crate) struct XPathEvaluator {
    reflector_: Reflector,
    window: Dom<Window>,
}

impl XPathEvaluator {
    fn new_inherited(window: &Window) -> XPathEvaluator {
        XPathEvaluator {
            reflector_: Reflector::new(),
            window: Dom::from_ref(window),
        }
    }

    pub(crate) fn new(
        cx: &mut JSContext,
        window: &Window,
        proto: Option<HandleObject>,
    ) -> DomRoot<XPathEvaluator> {
        reflect_dom_object_with_proto_and_cx(
            Box::new(XPathEvaluator::new_inherited(window)),
            window,
            proto,
            cx,
        )
    }
}

impl XPathEvaluatorMethods<crate::DomTypeHolder> for XPathEvaluator {
    /// <https://dom.spec.whatwg.org/#dom-xpathevaluator-xpathevaluator>
    fn Constructor(
        cx: &mut JSContext,
        window: &Window,
        proto: Option<HandleObject>,
    ) -> DomRoot<XPathEvaluator> {
        XPathEvaluator::new(cx, window, proto)
    }

    /// <https://dom.spec.whatwg.org/#dom-xpathevaluatorbase-createexpression>
    fn CreateExpression(
        &self,
        cx: &mut JSContext,
        expression: DOMString,
        resolver: Option<Rc<XPathNSResolver>>,
    ) -> Fallible<DomRoot<XPathExpression>> {
        let parsed_expression = parse_expression(
            &expression.str(),
            resolver,
            self.window.Document().is_html_document(),
        )?;
        Ok(XPathExpression::new(
            cx,
            &self.window,
            None,
            parsed_expression,
        ))
    }

    /// <https://dom.spec.whatwg.org/#dom-xpathevaluatorbase-creatensresolver>
    fn CreateNSResolver(&self, node_resolver: &Node) -> DomRoot<Node> {
        // Legacy: the spec tells us to just return `node_resolver` as-is
        DomRoot::from_ref(node_resolver)
    }

    /// <https://dom.spec.whatwg.org/#dom-xpathevaluatorbase-evaluate>
    fn Evaluate(
        &self,
        cx: &mut JSContext,
        expression: DOMString,
        context_node: &Node,
        resolver: Option<Rc<XPathNSResolver>>,
        result_type: u16,
        result: Option<&XPathResult>,
    ) -> Fallible<DomRoot<XPathResult>> {
        let parsed_expression = parse_expression(
            &expression.str(),
            resolver,
            self.window.Document().is_html_document(),
        )?;
        let expression = XPathExpression::new(cx, &self.window, None, parsed_expression);
        expression.evaluate_internal(cx, context_node, result_type, result)
    }
}
