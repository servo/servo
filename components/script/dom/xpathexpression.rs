/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::rust::HandleObject;

use super::types::XPathResult;
use crate::dom::bindings::codegen::Bindings::XPathExpressionBinding::XPathExpressionMethods;
use crate::dom::bindings::error::{Error, Fallible};
use crate::dom::bindings::reflector::{reflect_dom_object_with_proto, DomObject, Reflector};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::node::Node;
use crate::dom::window::Window;
use crate::dom::xpathresult::XPathResultType;
use crate::script_runtime::CanGc;
use crate::xpath::{evaluate_parsed_xpath, Expr, Value};

#[dom_struct]
pub struct XPathExpression {
    reflector_: Reflector,
    window: Dom<Window>,
    #[no_trace]
    parsed_expression: Option<Expr>,
}

impl XPathExpression {
    fn new_inherited(window: &Window, parsed_expression: Option<Expr>) -> XPathExpression {
        XPathExpression {
            reflector_: Reflector::new(),
            window: Dom::from_ref(window),
            parsed_expression,
        }
    }

    pub fn new(
        window: &Window,
        proto: Option<HandleObject>,
        can_gc: CanGc,
        parsed_expression: Option<Expr>,
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
    fn Evaluate(
        &self,
        context_node: &Node,
        result_type_num: u16,
        _result: Option<&super::types::XPathResult>,
    ) -> Fallible<DomRoot<super::types::XPathResult>> {
        let result_type = XPathResultType::try_from(result_type_num)
            .map_err(|()| Error::Type("Invalid XPath result type".to_string()))?;

        let global = self.global();
        let window = global.as_window();

        let result_value = if let Some(ref parsed_expression) = self.parsed_expression {
            evaluate_parsed_xpath(parsed_expression, context_node).map_err(|_e| Error::Operation)?
        } else {
            Value::Nodeset(vec![])
        };

        // TODO(vlindhol): support putting results into mutable `_result` as per the spec
        Ok(XPathResult::new(
            window,
            None,
            CanGc::note(),
            result_type,
            result_value.into(),
        ))
    }
}
