/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use webnn::graph::{GraphNode, InputInfo};
use webnn::model::{CompiledModel, RunResult};
use webnn::traits::Backend;
use webnn::types::{DataType, TensorDesc};

#[test]
fn test_data_type_byte_size() {
    assert_eq!(DataType::Float32.element_byte_size(), 4);
    assert_eq!(DataType::Float16.element_byte_size(), 2);
    assert_eq!(DataType::Int32.element_byte_size(), 4);
    assert_eq!(DataType::Int8.element_byte_size(), 1);
}

#[test]
fn test_data_type_from_u32() {
    assert_eq!(DataType::from_u32(0), DataType::Float32);
    assert_eq!(DataType::from_u32(7), DataType::Uint8);
}

#[test]
fn test_tensor_desc() {
    let desc = TensorDesc {
        data_type: DataType::Float32,
        shape: vec![1, 2, 3, 4],
    };
    assert_eq!(desc.num_elements(), 24);
    assert_eq!(desc.byte_length(), 96);
}

struct MockBackend;

impl Backend for MockBackend {
    fn name(&self) -> &'static str {
        "mock"
    }

    fn compile(
        &self,
        _nodes: &[GraphNode],
        _input_infos: &[InputInfo],
        _output_names: &[String],
    ) -> Result<CompiledModel, String> {
        Ok(CompiledModel(Box::new(4usize)))
    }

    fn run(&self, _model: &CompiledModel, _inputs: &[(&str, &[u8])]) -> Result<RunResult, String> {
        Ok(RunResult {
            outputs: vec![vec![0u8; 4]],
        })
    }
}

#[test]
fn test_backend_trait_object_safe() {
    let backend: Box<dyn Backend> = Box::new(MockBackend);
    assert_eq!(backend.name(), "mock");
}
