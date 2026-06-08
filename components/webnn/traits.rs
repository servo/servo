/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::graph::{GraphNode, InputInfo};
use crate::model::{CompiledModel, RunResult};

/// A WebNN backend: compiles graphs and runs inference.
///
/// Must be object-safe (`Send + Sync`) so it can be stored in a `Box<dyn Backend>`.
pub trait Backend: Send + Sync {
    fn name(&self) -> &'static str;

    fn compile(
        &self,
        nodes: &[GraphNode],
        input_infos: &[InputInfo],
        output_names: &[String],
    ) -> Result<CompiledModel, String>;

    fn run(&self, model: &CompiledModel, inputs: &[(&str, &[u8])]) -> Result<RunResult, String>;
}
