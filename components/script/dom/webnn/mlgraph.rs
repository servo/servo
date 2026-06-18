/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;
use std::collections::HashMap;
use std::rc::Rc;

use dom_struct::dom_struct;
use js::context::JSContext;
use script_bindings::cell::DomRefCell;
use script_bindings::reflector::{Reflector, reflect_dom_object_with_cx};

use crate::dom::bindings::codegen::Bindings::WebNNBinding::{MLGraphMethods, MLOperandDataType};
use crate::dom::bindings::error::Error;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::weakref::WeakRef;
use crate::dom::globalscope::GlobalScope;
use crate::dom::promise::Promise;
use crate::dom::webnn::mlcontext::MLContext;
use crate::routed_promise::RoutedPromiseListener;

/// <https://www.w3.org/TR/webnn/#mlgraph>
#[dom_struct]
pub(crate) struct MLGraph {
    reflector_: Reflector,
    /// <https://www.w3.org/TR/webnn/#dom-mlgraphbuilder-context-slot>
    context: WeakRef<MLContext>,
    graph_id: Cell<usize>,
    /// <https://www.w3.org/TR/webnn/#dom-mlgraph-isdestroyed-slot>
    is_destroyed: Cell<bool>,
    /// <https://www.w3.org/TR/webnn/#dom-mlgraph-outputdescriptors-slot>
    output_descriptors: DomRefCell<HashMap<String, (MLOperandDataType, Vec<u32>)>>,
}

impl MLGraph {
    pub(crate) fn new_inherited(context: &MLContext) -> MLGraph {
        MLGraph {
            reflector_: Reflector::new(),
            context: WeakRef::new(context),
            graph_id: Cell::new(0),
            is_destroyed: Cell::new(false),
            output_descriptors: DomRefCell::new(HashMap::new()),
        }
    }

    pub(crate) fn new(
        global: &GlobalScope,
        context: &MLContext,
        cx: &mut JSContext,
    ) -> DomRoot<MLGraph> {
        reflect_dom_object_with_cx(Box::new(MLGraph::new_inherited(context)), global, cx)
    }

    /// <https://www.w3.org/TR/webnn/#dom-mlgraph-context-slot>
    pub(crate) fn context(&self) -> &WeakRef<MLContext> {
        &self.context
    }

    /// <https://www.w3.org/TR/webnn/#dom-mlgraph-isdestroyed-slot>
    pub(crate) fn is_destroyed(&self) -> bool {
        self.is_destroyed.get()
    }

    pub(crate) fn graph_id(&self) -> usize {
        self.graph_id.get()
    }

    pub(crate) fn set_graph_id(&self, graph_id: usize) {
        self.graph_id.set(graph_id);
    }

    /// <https://www.w3.org/TR/webnn/#dom-mlgraph-outputdescriptors-slot>
    pub(crate) fn set_output_descriptors(
        &self,
        descriptors: HashMap<String, (MLOperandDataType, Vec<u32>)>,
    ) {
        *self.output_descriptors.borrow_mut() = descriptors;
    }

    /// <https://www.w3.org/TR/webnn/#dom-mlgraph-outputdescriptors-slot>
    #[allow(dead_code)]
    pub(crate) fn output_descriptors(
        &self,
    ) -> std::cell::Ref<'_, HashMap<String, (MLOperandDataType, Vec<u32>)>> {
        self.output_descriptors.borrow()
    }
}

impl MLGraphMethods<crate::DomTypeHolder> for MLGraph {
    /// <https://www.w3.org/TR/webnn/#dom-mlgraph-destroy>
    fn Destroy(&self) {
        // Step 1. If [[isDestroyed]] is true, then return.
        if self.is_destroyed.get() {
            return;
        }
        // Step 2. Set [[isDestroyed]] to true.
        self.is_destroyed.set(true);
        // Step 3. Queue a task on this.[[context]].[[timeline]] to mark
        // resources owned by this graph as freeable.
        if let Some(context) = self.context.root() {
            context.channel().destroy_graph(self.graph_id.get());
        }
    }
}

impl RoutedPromiseListener<webnn::BuildResponse> for MLGraph {
    fn handle_response(
        &self,
        cx: &mut JSContext,
        response: webnn::BuildResponse,
        promise: &Rc<Promise>,
    ) {
        match response.graph_id {
            Ok(graph_id) => {
                self.set_graph_id(graph_id);
                promise.resolve_native(cx, self);
            },
            Err(msg) => {
                promise.reject_error(cx, Error::Operation(Some(msg)));
            },
        }
    }
}
