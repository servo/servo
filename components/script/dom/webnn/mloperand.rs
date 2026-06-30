/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::context::JSContext;
use js::conversions::ToJSValConvertible;
use js::rust::MutableHandleValue;
use script_bindings::reflector::{Reflector, reflect_dom_object_with_cx};
use script_bindings::root::DomRoot;

use crate::dom::bindings::codegen::Bindings::WebNNBinding::{MLOperandDataType, MLOperandMethods};
use crate::dom::bindings::weakref::WeakRef;
use crate::dom::globalscope::GlobalScope;
use crate::dom::webnn::mlgraphbuilder::MLGraphBuilder;

/// <https://www.w3.org/TR/webnn/#mloperand>
#[dom_struct]
pub(crate) struct MLOperand {
    reflector_: Reflector,
    /// Backend-assigned identifier for this node.
    operand_id: webnn::OperandId,
    /// <https://www.w3.org/TR/webnn/#mloperand-datatype>
    data_type: MLOperandDataType,
    /// <https://www.w3.org/TR/webnn/#mloperand-shape>
    shape: Vec<u32>,
    /// <https://www.w3.org/TR/webnn/#dom-mloperand-builder-slot>
    builder: WeakRef<MLGraphBuilder>,
}

impl MLOperand {
    pub(crate) fn new_inherited(
        operand_id: webnn::OperandId,
        data_type: MLOperandDataType,
        shape: Vec<u32>,
        builder: &MLGraphBuilder,
    ) -> MLOperand {
        MLOperand {
            reflector_: Reflector::new(),
            operand_id,
            data_type,
            shape,
            builder: WeakRef::new(builder),
        }
    }

    pub(crate) fn new(
        global: &GlobalScope,
        operand_id: webnn::OperandId,
        data_type: MLOperandDataType,
        shape: Vec<u32>,
        builder: &MLGraphBuilder,
        cx: &mut JSContext,
    ) -> DomRoot<MLOperand> {
        reflect_dom_object_with_cx(
            Box::new(MLOperand::new_inherited(
                operand_id, data_type, shape, builder,
            )),
            global,
            cx,
        )
    }

    pub(crate) fn operand_id(&self) -> webnn::OperandId {
        self.operand_id
    }

    /// <https://www.w3.org/TR/webnn/#dom-mloperand-datatype>
    pub(crate) fn data_type(&self) -> MLOperandDataType {
        self.data_type
    }

    /// <https://www.w3.org/TR/webnn/#dom-mloperand-shape>
    pub(crate) fn shape(&self) -> &[u32] {
        &self.shape
    }

    /// <https://www.w3.org/TR/webnn/#dom-mloperand-builder-slot>
    pub(crate) fn builder(&self) -> &WeakRef<MLGraphBuilder> {
        &self.builder
    }
}

impl MLOperandMethods<crate::DomTypeHolder> for MLOperand {
    /// <https://www.w3.org/TR/webnn/#dom-mloperand-datatype>
    fn DataType(&self) -> MLOperandDataType {
        // > The dataType getter steps are to return this's dataType.
        self.data_type
    }

    /// <https://www.w3.org/TR/webnn/#dom-mloperand-shape>
    #[allow(unsafe_code)]
    fn Shape(&self, cx: &mut JSContext, retval: MutableHandleValue) {
        // > The shape getter steps are to return this's shape.
        let js_shape: Vec<f64> = self
            .shape
            .iter()
            .map(|&dimension| dimension as f64)
            .collect();
        unsafe { js_shape.to_jsval(cx.raw_cx(), retval) }
    }
}
