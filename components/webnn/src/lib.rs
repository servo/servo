/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use servo_base::generic_channel::{self, GenericCallback, GenericOneshotSender, GenericSender};

mod mock_backend;

pub use mock_backend::MockBackend;

pub type GraphId = usize;
pub type BuilderId = usize;
pub type OperandId = usize;

// ── Operator option values ──

/// A value that can be passed as an operator option.
/// <https://www.w3.org/TR/webnn/#dom-mloperatoroptions>
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum OperatorOptionValue {
    /// <https://www.w3.org/TR/webnn/#dom-mlgemmoptions-alpha>
    F64(f64),
    /// <https://www.w3.org/TR/webnn/#dom-mlgemmoptions-atranspose>
    Bool(bool),
    /// <https://www.w3.org/TR/webnn/#dom-mlconv2doptions-inputlayout>
    String(String),
    /// <https://www.w3.org/TR/webnn/#dom-mlconv2doptions-padding>
    U32(Vec<u32>),
    /// <https://www.w3.org/TR/webnn/#dom-mlresample2doptions-scales>
    F32(Vec<f32>),
    /// <https://www.w3.org/TR/webnn/#dom-mlconv2doptions-bias>
    Operand(OperandId),
}

pub type OperatorOptions = HashMap<String, OperatorOptionValue>;

#[derive(Clone, Serialize, Deserialize)]
pub struct RunResult {
    pub outputs: Vec<Vec<u8>>,
}

// ── Backend trait ──

pub trait Backend: Send + 'static {
    fn name(&self) -> &str;
    fn create_builder(&self) -> BuilderId;
    fn add_input(
        &self,
        builder_id: BuilderId,
        name: &str,
        data_type: u32,
        shape: &[u32],
    ) -> OperandId;
    fn add_constant(
        &self,
        builder_id: BuilderId,
        data_type: u32,
        shape: &[u32],
        data: &[u8],
    ) -> OperandId;
    fn add_operator(
        &self,
        builder_id: BuilderId,
        op: &str,
        inputs: &[OperandId],
        data_type: u32,
        shape: &[u32],
        options: &OperatorOptions,
        label: &str,
    ) -> OperandId;
    fn build(
        &self,
        builder_id: BuilderId,
        outputs: &[(String, OperandId)],
    ) -> Result<GraphId, String>;
    fn run(
        &self,
        graph_id: GraphId,
        inputs: &[(String, &[u8])],
        output_labels: &[String],
    ) -> Result<RunResult, String>;
    fn destroy_graph(&self, graph_id: GraphId);
}

// ── Async responses (backend -> script thread via GenericCallback) ──

#[derive(Serialize, Deserialize)]
pub struct BuildResponse {
    pub graph_id: Result<GraphId, String>,
}

#[derive(Serialize, Deserialize)]
pub struct RunResponse {
    pub result: Result<RunResult, String>,
}

// ── Thread requests ──

#[derive(Serialize, Deserialize)]
enum WebNNRequest {
    CreateBuilder(GenericOneshotSender<BuilderId>),
    AddInput {
        builder_id: BuilderId,
        name: String,
        data_type: u32,
        shape: Vec<u32>,
        reply: GenericOneshotSender<OperandId>,
    },
    AddConstant {
        builder_id: BuilderId,
        data_type: u32,
        shape: Vec<u32>,
        data: Vec<u8>,
        reply: GenericOneshotSender<OperandId>,
    },
    AddOperator {
        builder_id: BuilderId,
        op: String,
        inputs: Vec<OperandId>,
        data_type: u32,
        shape: Vec<u32>,
        options: OperatorOptions,
        label: String,
        reply: GenericOneshotSender<OperandId>,
    },
    Build {
        builder_id: BuilderId,
        outputs: Vec<(String, OperandId)>,
        callback: GenericCallback<BuildResponse>,
    },
    Run {
        graph_id: GraphId,
        inputs: Vec<(String, Vec<u8>)>,
        output_labels: Vec<String>,
        callback: GenericCallback<RunResponse>,
    },
    DestroyGraph {
        graph_id: GraphId,
    },
    #[allow(dead_code)]
    Shutdown,
}

// ── WebNN channel ──

/// Channel from script thread to the WebNN backend thread.
/// Every `MLContext` holds a clone
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct WebNN(pub(crate) GenericSender<WebNNRequest>);

impl WebNN {
    /// Spawn the WebNN thread with the given backend.
    pub fn new<B: Backend>(backend: B) -> Self {
        let (sender, receiver) =
            generic_channel::channel::<WebNNRequest>().expect("WebNN channel creation");
        std::thread::Builder::new()
            .name("WebNN".into())
            .spawn(move || {
                while let Ok(request) = receiver.recv() {
                    match request {
                        WebNNRequest::CreateBuilder(reply) => {
                            let id = backend.create_builder();
                            reply.send_or_warn(id);
                        },
                        WebNNRequest::AddInput {
                            builder_id,
                            name,
                            data_type,
                            shape,
                            reply,
                        } => {
                            let id = backend.add_input(builder_id, &name, data_type, &shape);
                            reply.send_or_warn(id);
                        },
                        WebNNRequest::AddConstant {
                            builder_id,
                            data_type,
                            shape,
                            data,
                            reply,
                        } => {
                            let id = backend.add_constant(builder_id, data_type, &shape, &data);
                            reply.send_or_warn(id);
                        },
                        WebNNRequest::AddOperator {
                            builder_id,
                            op,
                            inputs,
                            data_type,
                            shape,
                            options,
                            label,
                            reply,
                        } => {
                            let id = backend.add_operator(
                                builder_id, &op, &inputs, data_type, &shape, &options, &label,
                            );
                            reply.send_or_warn(id);
                        },
                        WebNNRequest::Build {
                            builder_id,
                            outputs,
                            callback,
                        } => {
                            let result = backend.build(builder_id, &outputs);
                            let _ = callback.send(BuildResponse { graph_id: result });
                        },
                        WebNNRequest::Run {
                            graph_id,
                            inputs,
                            output_labels,
                            callback,
                        } => {
                            let input_refs: Vec<(String, &[u8])> = inputs
                                .iter()
                                .map(|(n, d)| (n.clone(), d.as_slice()))
                                .collect();
                            let result = backend.run(graph_id, &input_refs, &output_labels);
                            let _ = callback.send(RunResponse { result });
                        },
                        WebNNRequest::DestroyGraph { graph_id } => {
                            backend.destroy_graph(graph_id);
                        },
                        WebNNRequest::Shutdown => break,
                    }
                }
            })
            .expect("WebNN thread spawn");
        WebNN(sender)
    }

    pub fn create_builder(&self) -> BuilderId {
        let (tx, rx) = generic_channel::oneshot().expect("WebNN oneshot");
        self.0.send_or_warn(WebNNRequest::CreateBuilder(tx));
        rx.recv().unwrap_or(0)
    }

    pub fn add_input(
        &self,
        builder_id: BuilderId,
        name: &str,
        data_type: u32,
        shape: &[u32],
    ) -> OperandId {
        let (tx, rx) = generic_channel::oneshot().expect("WebNN oneshot");
        self.0.send_or_warn(WebNNRequest::AddInput {
            builder_id,
            name: name.to_string(),
            data_type,
            shape: shape.to_vec(),
            reply: tx,
        });
        rx.recv().unwrap_or(0)
    }

    pub fn add_constant(
        &self,
        builder_id: BuilderId,
        data_type: u32,
        shape: &[u32],
        data: &[u8],
    ) -> OperandId {
        let (tx, rx) = generic_channel::oneshot().expect("WebNN oneshot");
        self.0.send_or_warn(WebNNRequest::AddConstant {
            builder_id,
            data_type,
            shape: shape.to_vec(),
            data: data.to_vec(),
            reply: tx,
        });
        rx.recv().unwrap_or(0)
    }

    pub fn add_operator(
        &self,
        builder_id: BuilderId,
        op: &str,
        inputs: &[OperandId],
        data_type: u32,
        shape: &[u32],
        options: &OperatorOptions,
        label: &str,
    ) -> OperandId {
        let (tx, rx) = generic_channel::oneshot().expect("WebNN oneshot");
        self.0.send_or_warn(WebNNRequest::AddOperator {
            builder_id,
            op: op.to_string(),
            inputs: inputs.to_vec(),
            data_type,
            shape: shape.to_vec(),
            options: options.clone(),
            label: label.to_string(),
            reply: tx,
        });
        rx.recv().unwrap_or(0)
    }

    pub fn build(
        &self,
        builder_id: BuilderId,
        outputs: &[(String, OperandId)],
        callback: GenericCallback<BuildResponse>,
    ) {
        self.0.send_or_warn(WebNNRequest::Build {
            builder_id,
            outputs: outputs.to_vec(),
            callback,
        });
    }

    pub fn run(
        &self,
        graph_id: GraphId,
        inputs: &[(String, &[u8])],
        output_labels: &[String],
        callback: GenericCallback<RunResponse>,
    ) {
        self.0.send_or_warn(WebNNRequest::Run {
            graph_id,
            inputs: inputs
                .iter()
                .map(|(n, d)| (n.to_string(), d.to_vec()))
                .collect(),
            output_labels: output_labels.to_vec(),
            callback,
        });
    }

    pub fn destroy_graph(&self, graph_id: GraphId) {
        self.0.send_or_warn(WebNNRequest::DestroyGraph { graph_id });
    }
}
