/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::collections::HashMap;
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};

use crate::{Backend, BuilderId, GraphId, OperandId, OperatorOptions, RunResult};

#[allow(dead_code)]
struct Node {
    op: String,
    inputs: Vec<OperandId>,
    data_type: u32,
    shape: Vec<u32>,
    data: Option<Vec<u8>>,
    label: String,
}

struct BuilderState {
    nodes: HashMap<OperandId, Node>,
    input_names: HashMap<String, OperandId>,
}

struct GraphState {
    #[allow(dead_code)]
    outputs: HashMap<String, OperandId>,
}

pub struct MockBackend {
    builders: Mutex<HashMap<BuilderId, BuilderState>>,
    graphs: Mutex<HashMap<GraphId, GraphState>>,
    next_builder_id: AtomicUsize,
    next_graph_id: AtomicUsize,
    next_operand_id: AtomicUsize,
}

impl MockBackend {
    pub fn new() -> Self {
        Self {
            builders: Mutex::new(HashMap::new()),
            graphs: Mutex::new(HashMap::new()),
            next_builder_id: AtomicUsize::new(1),
            next_graph_id: AtomicUsize::new(1),
            next_operand_id: AtomicUsize::new(1),
        }
    }

    fn next_operand_id(&self) -> OperandId {
        self.next_operand_id.fetch_add(1, Ordering::Relaxed)
    }
}

impl Default for MockBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl Backend for MockBackend {
    fn name(&self) -> &str {
        "mock"
    }

    fn create_builder(&self) -> BuilderId {
        let id = self.next_builder_id.fetch_add(1, Ordering::Relaxed);
        self.builders.lock().unwrap().insert(
            id,
            BuilderState {
                nodes: HashMap::new(),
                input_names: HashMap::new(),
            },
        );
        id
    }

    fn add_input(
        &self,
        builder_id: BuilderId,
        name: &str,
        data_type: u32,
        shape: &[u32],
    ) -> OperandId {
        let operand_id = self.next_operand_id();
        if let Some(state) = self.builders.lock().unwrap().get_mut(&builder_id) {
            state.input_names.insert(name.to_string(), operand_id);
            state.nodes.insert(
                operand_id,
                Node {
                    op: "input".to_string(),
                    inputs: Vec::new(),
                    data_type,
                    shape: shape.to_vec(),
                    data: None,
                    label: String::new(),
                },
            );
        }
        operand_id
    }

    fn add_constant(
        &self,
        builder_id: BuilderId,
        data_type: u32,
        shape: &[u32],
        data: &[u8],
    ) -> OperandId {
        let operand_id = self.next_operand_id();
        if let Some(state) = self.builders.lock().unwrap().get_mut(&builder_id) {
            state.nodes.insert(
                operand_id,
                Node {
                    op: "constant".to_string(),
                    inputs: Vec::new(),
                    data_type,
                    shape: shape.to_vec(),
                    data: Some(data.to_vec()),
                    label: String::new(),
                },
            );
        }
        operand_id
    }

    fn add_operator(
        &self,
        builder_id: BuilderId,
        op: &str,
        inputs: &[OperandId],
        data_type: u32,
        shape: &[u32],
        _options: &OperatorOptions,
        label: &str,
    ) -> OperandId {
        let operand_id = self.next_operand_id();
        if let Some(state) = self.builders.lock().unwrap().get_mut(&builder_id) {
            state.nodes.insert(
                operand_id,
                Node {
                    op: op.to_string(),
                    inputs: inputs.to_vec(),
                    data_type,
                    shape: shape.to_vec(),
                    data: None,
                    label: label.to_string(),
                },
            );
        }
        operand_id
    }

    fn build(
        &self,
        builder_id: BuilderId,
        outputs: &[(String, OperandId)],
    ) -> Result<GraphId, String> {
        let id = self.next_graph_id.fetch_add(1, Ordering::Relaxed);
        let output_map: HashMap<String, OperandId> = outputs.iter().cloned().collect();
        self.graphs.lock().unwrap().insert(
            id,
            GraphState {
                outputs: output_map,
            },
        );
        self.builders.lock().unwrap().remove(&builder_id);
        Ok(id)
    }

    fn run(
        &self,
        _graph_id: GraphId,
        inputs: &[(String, &[u8])],
        output_labels: &[String],
    ) -> Result<RunResult, String> {
        let data = inputs.first().map(|(_, d)| d.to_vec()).unwrap_or_default();
        let count = output_labels.len().max(1);
        Ok(RunResult {
            outputs: vec![data; count],
        })
    }

    fn destroy_graph(&self, graph_id: GraphId) {
        self.graphs.lock().unwrap().remove(&graph_id);
    }
}
