/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

/// WebNN operand data type.
///
/// <https://www.w3.org/TR/webnn/#enumdef-mloperanddatatype>
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DataType {
    Float32 = 0,
    Float16 = 1,
    Int32 = 2,
    Uint32 = 3,
    Int64 = 4,
    Uint64 = 5,
    Int8 = 6,
    Uint8 = 7,
}

impl DataType {
    pub fn from_u32(v: u32) -> Self {
        match v {
            0 => DataType::Float32,
            1 => DataType::Float16,
            2 => DataType::Int32,
            3 => DataType::Uint32,
            4 => DataType::Int64,
            5 => DataType::Uint64,
            6 => DataType::Int8,
            7 => DataType::Uint8,
            _ => panic!("invalid DataType value: {v}"),
        }
    }

    pub fn element_byte_size(&self) -> usize {
        match self {
            DataType::Float32 | DataType::Int32 | DataType::Uint32 => 4,
            DataType::Float16 => 2,
            DataType::Int64 | DataType::Uint64 => 8,
            DataType::Int8 | DataType::Uint8 => 1,
        }
    }
}

/// Describes the shape and data type of a tensor operand.
///
/// <https://www.w3.org/TR/webnn/#dictdef-mloperanddescriptor>
#[derive(Clone, Debug)]
pub struct TensorDesc {
    pub data_type: DataType,
    pub shape: Vec<u32>,
}

impl TensorDesc {
    pub fn num_elements(&self) -> usize {
        self.shape.iter().map(|&d| d as usize).product()
    }

    pub fn byte_length(&self) -> usize {
        self.num_elements() * self.data_type.element_byte_size()
    }
}
