/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::rc::Rc;

use dom_struct::dom_struct;
use js::rust::HandleObject;

use super::bindings::error::Error;
use crate::dom::bindings::codegen::Bindings::XPathEvaluatorBinding::XPathEvaluatorMethods;
use crate::dom::bindings::codegen::Bindings::XPathExpressionBinding::XPathExpression_Binding::XPathExpressionMethods;
use crate::dom::bindings::codegen::Bindings::XPathNSResolverBinding::XPathNSResolver;
use crate::dom::bindings::error::Fallible;
use crate::dom::bindings::reflector::{reflect_dom_object_with_proto, DomObject, Reflector};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::DOMString;
use crate::dom::node::Node;
use crate::dom::window::Window;
use crate::dom::xpathexpression::XPathExpression;
use crate::dom::xpathresult::XPathResult;
use crate::script_runtime::CanGc;

#[dom_struct]
pub struct XPathEvaluator {
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

    pub fn new(
        window: &Window,
        proto: Option<HandleObject>,
        can_gc: CanGc,
    ) -> DomRoot<XPathEvaluator> {
        reflect_dom_object_with_proto(
            Box::new(XPathEvaluator::new_inherited(window)),
            window,
            proto,
            can_gc,
        )
    }
}

impl XPathEvaluatorMethods<crate::DomTypeHolder> for XPathEvaluator {
    fn Constructor(
        window: &Window,
        proto: Option<HandleObject>,
        can_gc: CanGc,
    ) -> DomRoot<XPathEvaluator> {
        XPathEvaluator::new(window, proto, can_gc)
    }

    fn CreateExpression(
        &self,
        expression: DOMString,
        _resolver: Option<Rc<XPathNSResolver>>,
    ) -> DomRoot<XPathExpression> {
        let global = self.global();
        let window = global.as_window();
        // NB: this function is *not* Fallible according to the spec, so we swallow any parsing errors and
        // just pass a None as the expression... it's not great.
        let parsed_expression = crate::xpath::parse(&expression);
        XPathExpression::new(window, None, CanGc::note(), parsed_expression.ok())
    }

    // Legacy: the spec tells us to just return `node_resolver` as-is
    fn CreateNSResolver(&self, node_resolver: &Node) -> DomRoot<Node> {
        DomRoot::from_ref(node_resolver)
    }

    fn Evaluate(
        &self,
        expression_str: DOMString,
        context_node: &Node,
        _resolver: Option<Rc<XPathNSResolver>>,
        result_type: u16,
        result: Option<&XPathResult>,
    ) -> Fallible<DomRoot<XPathResult>> {
        let global = self.global();
        let window = global.as_window();
        let parsed_expression = crate::xpath::parse(&expression_str).map_err(|_| Error::Syntax)?;
        let expression = XPathExpression::new(window, None, CanGc::note(), Some(parsed_expression));
        XPathExpressionMethods::<crate::DomTypeHolder>::Evaluate(
            &*expression,
            context_node,
            result_type,
            result,
        )
    }
}
