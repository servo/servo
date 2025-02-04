/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::rust::HandleObject;

use crate::dom::bindings::codegen::Bindings::XPathExpressionBinding::XPathExpressionMethods;
use crate::dom::bindings::error::{Error, Fallible};
use crate::dom::bindings::reflector::{reflect_dom_object_with_proto, DomGlobal, Reflector};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::node::Node;
use crate::dom::window::Window;
use crate::dom::xpathresult::{XPathResult, XPathResultType};
use crate::script_runtime::CanGc;
use crate::xpath::{evaluate_parsed_xpath, Expr};

#[dom_struct]
pub(crate) struct XPathExpression {
    reflector_: Reflector,
    window: Dom<Window>,
    #[no_trace]
    parsed_expression: Expr,
}

impl XPathExpression {
    fn new_inherited(window: &Window, parsed_expression: Expr) -> XPathExpression {
        XPathExpression {
            reflector_: Reflector::new(),
            window: Dom::from_ref(window),
            parsed_expression,
        }
    }

    pub(crate) fn new(
        window: &Window,
        proto: Option<HandleObject>,
        can_gc: CanGc,
        parsed_expression: Expr,
    ) -> DomRoot<XPathExpression> {
        reflect_dom_object_with_proto(
            Box::new(XPathExpression::new_inherited(window, parsed_expression)),
            window,
            proto,
            can_gc,
        )
    }
}

impl XPathExpressionMethods<crate::DomTypeHolder> for XPathExpression {
    /// <https://dom.spec.whatwg.org/#dom-xpathexpression-evaluate>
    fn Evaluate(
        &self,
        context_node: &Node,
        result_type_num: u16,
        _result: Option<&XPathResult>,
        can_gc: CanGc,
    ) -> Fallible<DomRoot<XPathResult>> {
        let result_type = XPathResultType::try_from(result_type_num)
            .map_err(|()| Error::Type("Invalid XPath result type".to_string()))?;

        let global = self.global();
        let window = global.as_window();

        let result_value = evaluate_parsed_xpath(&self.parsed_expression, context_node)
            .map_err(|_e| Error::Operation)?;

        // TODO(vlindhol): support putting results into mutable `_result` as per the spec
        Ok(XPathResult::new(
            window,
            None,
            can_gc,
            result_type,
            result_value.into(),
        ))
    }
}
