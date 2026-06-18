/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use servo_base::generic_channel::{
    self, GenericCallback, GenericOneshotSender, GenericReceiver, GenericSender,
};
use servo_base::id::MLContextId;

mod mock_backend;

pub use mock_backend::MockBackend;

pub type GraphId = usize;
pub type BuilderId = usize;
pub type OperandId = usize;
/// Identifier of an `MLContext` on the shared WebNN backend thread.
pub type ContextId = MLContextId;

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

// ── Backend options ──

/// <https://www.w3.org/TR/webnn/#enumdef-mlpowerpreference>
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum BackendPowerPreference {
    Default,
    HighPerformance,
    LowPower,
}

/// Backend-agnostic subset of `MLContextOptions`, used to select a backend.
/// <https://www.w3.org/TR/webnn/#dictdef-mlcontextoptions>
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BackendOptions {
    pub power_preference: BackendPowerPreference,
    pub accelerated: bool,
}

// ── Backend trait ──

pub trait Backend: Send + 'static {
    fn name(&self) -> &str;
    fn create_builder(&self) -> BuilderId;
    fn add_input(
        &self,
        builder_id: BuilderId,
        operand_id: OperandId,
        name: &str,
        data_type: u32,
        shape: &[u32],
    );
    fn add_constant(
        &self,
        builder_id: BuilderId,
        operand_id: OperandId,
        data_type: u32,
        shape: &[u32],
        data: &[u8],
    );
    fn add_operator(
        &self,
        builder_id: BuilderId,
        operand_id: OperandId,
        operator: &str,
        inputs: &[OperandId],
        data_type: u32,
        shape: &[u32],
        options: &OperatorOptions,
        label: &str,
    );
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

// ── Backend selection ──

/// Select a backend for a new context.
///
/// This is the seam later backends (`RustnnBackend`, etc.) plug into; for now
/// it always returns the mock backend.
pub fn create_backend(_options: &BackendOptions) -> Box<dyn Backend> {
    Box::new(MockBackend::new())
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
    NewContext {
        context_id: ContextId,
        options: BackendOptions,
    },
    DestroyContext {
        context_id: ContextId,
    },
    CreateBuilder {
        context_id: ContextId,
        reply: GenericOneshotSender<BuilderId>,
    },
    AddInput {
        context_id: ContextId,
        builder_id: BuilderId,
        operand_id: OperandId,
        name: String,
        data_type: u32,
        shape: Vec<u32>,
    },
    AddConstant {
        context_id: ContextId,
        builder_id: BuilderId,
        operand_id: OperandId,
        data_type: u32,
        shape: Vec<u32>,
        data: Vec<u8>,
    },
    AddOperator {
        context_id: ContextId,
        builder_id: BuilderId,
        operand_id: OperandId,
        operator: String,
        inputs: Vec<OperandId>,
        data_type: u32,
        shape: Vec<u32>,
        options: OperatorOptions,
        label: String,
    },
    Build {
        context_id: ContextId,
        builder_id: BuilderId,
        outputs: Vec<(String, OperandId)>,
        callback: GenericCallback<BuildResponse>,
    },
    Run {
        context_id: ContextId,
        graph_id: GraphId,
        inputs: Vec<(String, Vec<u8>)>,
        output_labels: Vec<String>,
        callback: GenericCallback<RunResponse>,
    },
    DestroyGraph {
        context_id: ContextId,
        graph_id: GraphId,
    },
    #[allow(dead_code)]
    Shutdown,
}

// ── WebNN channel ──

/// Channel from script thread to the shared WebNN backend thread.
/// A single backend thread hosts one backend per MLContext.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct WebNN(pub(crate) GenericSender<WebNNRequest>);

static WEBNN_CHANNEL: std::sync::OnceLock<WebNN> = std::sync::OnceLock::new();

impl WebNN {
    /// Get the shared WebNN thread, spawning it on first call.
    pub fn shared() -> &'static WebNN {
        WEBNN_CHANNEL.get_or_init(|| {
            let (sender, receiver) =
                generic_channel::channel::<WebNNRequest>().expect("WebNN channel creation");
            std::thread::Builder::new()
                .name("WebNN".into())
                .spawn(move || {
                    run_webnn_thread(receiver);
                })
                .expect("WebNN thread spawn");
            WebNN(sender)
        })
    }
}

/// Run the WebNN backend thread's main loop, routing requests to the backend
/// owned by each context.
fn run_webnn_thread(receiver: GenericReceiver<WebNNRequest>) {
    let mut backends: HashMap<ContextId, Box<dyn Backend>> = HashMap::new();
    while let Ok(request) = receiver.recv() {
        match request {
            WebNNRequest::NewContext {
                context_id,
                options,
            } => {
                backends.insert(context_id, create_backend(&options));
            },
            WebNNRequest::DestroyContext { context_id } => {
                backends.remove(&context_id);
            },
            WebNNRequest::CreateBuilder { context_id, reply } => {
                let id = backends
                    .get(&context_id)
                    .map(|backend| backend.create_builder())
                    .unwrap_or(0);
                reply.send_or_warn(id);
            },
            WebNNRequest::AddInput {
                context_id,
                builder_id,
                operand_id,
                name,
                data_type,
                shape,
            } => {
                if let Some(backend) = backends.get(&context_id) {
                    backend.add_input(builder_id, operand_id, &name, data_type, &shape);
                }
            },
            WebNNRequest::AddConstant {
                context_id,
                builder_id,
                operand_id,
                data_type,
                shape,
                data,
            } => {
                if let Some(backend) = backends.get(&context_id) {
                    backend.add_constant(builder_id, operand_id, data_type, &shape, &data);
                }
            },
            WebNNRequest::AddOperator {
                context_id,
                builder_id,
                operand_id,
                operator,
                inputs,
                data_type,
                shape,
                options,
                label,
            } => {
                if let Some(backend) = backends.get(&context_id) {
                    backend.add_operator(
                        builder_id, operand_id, &operator, &inputs, data_type, &shape, &options,
                        &label,
                    );
                }
            },
            WebNNRequest::Build {
                context_id,
                builder_id,
                outputs,
                callback,
            } => {
                let result = backends
                    .get(&context_id)
                    .map(|backend| backend.build(builder_id, &outputs))
                    .unwrap_or_else(|| Err("unknown context".to_string()));
                let _ = callback.send(BuildResponse { graph_id: result });
            },
            WebNNRequest::Run {
                context_id,
                graph_id,
                inputs,
                output_labels,
                callback,
            } => {
                let input_refs: Vec<(String, &[u8])> = inputs
                    .iter()
                    .map(|(n, d)| (n.clone(), d.as_slice()))
                    .collect();
                let result = backends
                    .get(&context_id)
                    .map(|backend| backend.run(graph_id, &input_refs, &output_labels))
                    .unwrap_or_else(|| Err("unknown context".to_string()));
                let _ = callback.send(RunResponse { result });
            },
            WebNNRequest::DestroyGraph {
                context_id,
                graph_id,
            } => {
                if let Some(backend) = backends.get(&context_id) {
                    backend.destroy_graph(graph_id);
                }
            },
            WebNNRequest::Shutdown => break,
        }
    }
}

impl WebNN {
    /// Register a new context and its backend on the shared thread.
    pub fn new_context(&self, context_id: ContextId, options: &BackendOptions) {
        self.0.send_or_warn(WebNNRequest::NewContext {
            context_id,
            options: options.clone(),
        });
    }

    /// Remove a context and its backend from the shared thread.
    pub fn destroy_context(&self, context_id: ContextId) {
        self.0
            .send_or_warn(WebNNRequest::DestroyContext { context_id });
    }

    pub fn create_builder(&self, context_id: ContextId) -> BuilderId {
        let (tx, rx) = generic_channel::oneshot().expect("WebNN oneshot");
        self.0.send_or_warn(WebNNRequest::CreateBuilder {
            context_id,
            reply: tx,
        });
        rx.recv().unwrap_or(0)
    }

    pub fn add_input(
        &self,
        context_id: ContextId,
        builder_id: BuilderId,
        operand_id: OperandId,
        name: &str,
        data_type: u32,
        shape: &[u32],
    ) {
        self.0.send_or_warn(WebNNRequest::AddInput {
            context_id,
            builder_id,
            operand_id,
            name: name.to_string(),
            data_type,
            shape: shape.to_vec(),
        });
    }

    pub fn add_constant(
        &self,
        context_id: ContextId,
        builder_id: BuilderId,
        operand_id: OperandId,
        data_type: u32,
        shape: &[u32],
        data: &[u8],
    ) {
        self.0.send_or_warn(WebNNRequest::AddConstant {
            context_id,
            builder_id,
            operand_id,
            data_type,
            shape: shape.to_vec(),
            data: data.to_vec(),
        });
    }

    pub fn add_operator(
        &self,
        context_id: ContextId,
        builder_id: BuilderId,
        operand_id: OperandId,
        operator: &str,
        inputs: &[OperandId],
        data_type: u32,
        shape: &[u32],
        options: &OperatorOptions,
        label: &str,
    ) {
        self.0.send_or_warn(WebNNRequest::AddOperator {
            context_id,
            builder_id,
            operand_id,
            operator: operator.to_string(),
            inputs: inputs.to_vec(),
            data_type,
            shape: shape.to_vec(),
            options: options.clone(),
            label: label.to_string(),
        });
    }

    pub fn build(
        &self,
        context_id: ContextId,
        builder_id: BuilderId,
        outputs: &[(String, OperandId)],
        callback: GenericCallback<BuildResponse>,
    ) {
        self.0.send_or_warn(WebNNRequest::Build {
            context_id,
            builder_id,
            outputs: outputs.to_vec(),
            callback,
        });
    }

    pub fn run(
        &self,
        context_id: ContextId,
        graph_id: GraphId,
        inputs: &[(String, &[u8])],
        output_labels: &[String],
        callback: GenericCallback<RunResponse>,
    ) {
        self.0.send_or_warn(WebNNRequest::Run {
            context_id,
            graph_id,
            inputs: inputs
                .iter()
                .map(|(n, d)| (n.to_string(), d.to_vec()))
                .collect(),
            output_labels: output_labels.to_vec(),
            callback,
        });
    }

    pub fn destroy_graph(&self, context_id: ContextId, graph_id: GraphId) {
        self.0.send_or_warn(WebNNRequest::DestroyGraph {
            context_id,
            graph_id,
        });
    }
}
