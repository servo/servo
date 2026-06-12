/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::collections::HashMap;

use crate::types::TensorDesc;

/// Operator attributes: name-value pairs (e.g. `{"stride": 2.0, "padding": 1.0}`).
pub type OpAttrs = HashMap<String, f64>;

/// A single computational node in the WebNN graph IR.
#[derive(Clone, Debug)]
pub struct GraphNode {
    pub op: String,
    pub inputs: Vec<String>,
    pub output: String,
    pub desc: TensorDesc,
    pub attrs: OpAttrs,
    pub data: Option<Vec<u8>>,
}

/// Metadata for a named graph input.
#[derive(Clone, Debug)]
pub struct InputInfo {
    pub name: String,
    pub shape: Vec<u32>,
    pub data_type: crate::types::DataType,
}
