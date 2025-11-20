/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::rust::HandleObject;
use script_bindings::codegen::InheritTypes::{CharacterDataTypeId, NodeTypeId};
use xpath::{Expression, evaluate_parsed_xpath};

use crate::dom::bindings::codegen::Bindings::XPathExpressionBinding::XPathExpressionMethods;
use crate::dom::bindings::error::{Error, Fallible};
use crate::dom::bindings::reflector::{DomGlobal, Reflector, reflect_dom_object_with_proto};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::node::Node;
use crate::dom::window::Window;
use crate::dom::xpathresult::{XPathResult, XPathResultType};
use crate::script_runtime::CanGc;
use crate::xpath::{Value, XPathImplementation};

#[dom_struct]
pub(crate) struct XPathExpression {
    reflector_: Reflector,
    window: Dom<Window>,
    #[no_trace]
    parsed_expression: Expression,
}

impl XPathExpression {
    fn new_inherited(window: &Window, parsed_expression: Expression) -> XPathExpression {
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
        parsed_expression: Expression,
    ) -> DomRoot<XPathExpression> {
        reflect_dom_object_with_proto(
            Box::new(XPathExpression::new_inherited(window, parsed_expression)),
            window,
            proto,
            can_gc,
        )
    }

    pub(crate) fn evaluate_internal(
        &self,
        context_node: &Node,
        result_type_num: u16,
        result: Option<&XPathResult>,
        can_gc: CanGc,
    ) -> Fallible<DomRoot<XPathResult>> {
        let is_allowed_context_node_type = matches!(
            context_node.type_id(),
            NodeTypeId::Attr |
                NodeTypeId::CharacterData(
                    CharacterDataTypeId::Comment |
                        CharacterDataTypeId::Text(_) |
                        CharacterDataTypeId::ProcessingInstruction
                ) |
                NodeTypeId::Document(_) |
                NodeTypeId::Element(_)
        );
        if !is_allowed_context_node_type {
            return Err(Error::NotSupported(None));
        }

        let result_type = XPathResultType::try_from(result_type_num)
            .map_err(|()| Error::Type("Invalid XPath result type".to_string()))?;

        let global = self.global();
        let window = global.as_window();

        let result_value = evaluate_parsed_xpath::<XPathImplementation>(
            &self.parsed_expression,
            DomRoot::from_ref(context_node).into(),
        )
        .map_err(|_| Error::Operation(None))?;

        // Cast the result to the type we wanted
        let result_value: Value = match result_type {
            XPathResultType::Boolean => result_value.convert_to_boolean().into(),
            XPathResultType::Number => result_value.convert_to_number().into(),
            XPathResultType::String => result_value.convert_to_string().into(),
            _ => result_value,
        };

        // TODO: if the wanted result type is AnyUnorderedNode | FirstOrderedNode,
        // we could drop all nodes except one to save memory.
        let inferred_result_type = if result_type == XPathResultType::Any {
            match result_value {
                Value::Boolean(_) => XPathResultType::Boolean,
                Value::Number(_) => XPathResultType::Number,
                Value::String(_) => XPathResultType::String,
                Value::NodeSet(_) => XPathResultType::UnorderedNodeIterator,
            }
        } else {
            result_type
        };

        if let Some(result) = result {
            // According to https://www.w3.org/TR/DOM-Level-3-XPath/xpath.html#XPathEvaluator-evaluate, reusing
            // the provided result object is optional. We choose to do it here because thats what other browsers do.
            result.reinitialize_with(inferred_result_type, result_value.into());
            Ok(DomRoot::from_ref(result))
        } else {
            Ok(XPathResult::new(
                window,
                None,
                can_gc,
                inferred_result_type,
                result_value.into(),
            ))
        }
    }
}

impl XPathExpressionMethods<crate::DomTypeHolder> for XPathExpression {
    /// <https://dom.spec.whatwg.org/#dom-xpathexpression-evaluate>
    fn Evaluate(
        &self,
        context_node: &Node,
        result_type_num: u16,
        result: Option<&XPathResult>,
        can_gc: CanGc,
    ) -> Fallible<DomRoot<XPathResult>> {
        self.evaluate_internal(context_node, result_type_num, result, can_gc)
    }
}
