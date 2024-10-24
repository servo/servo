/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Phony assignment execution tests
`;import { makeTestGroup } from '../../../../common/framework/test_group.js';
import { keysOf } from '../../../../common/util/data_tables.js';

import { GPUTest } from '../../../gpu_test.js';

export const g = makeTestGroup(GPUTest);

/**
 * Builds, runs then checks the output of a statement shader test.
 *
 * @param t The test object
 * @param ty The WGSL scalar type to be written
 * @param values The expected output values of type ty
 * @param wgsl_main The body of the WGSL entry point.
 */
export function runStatementTest(
t,
ty,
values,
wgsl_main)
{
  const wgsl = `
struct Outputs {
  data  : array<${ty}>,
};
var<private> count: u32 = 0;

@group(0) @binding(1) var<storage, read_write> outputs : Outputs;

fn put(value : ${ty}) -> ${ty} {
  outputs.data[count] = value;
  count += 1;
  return value;
}

@compute @workgroup_size(1)
fn main() {
  let x = outputs.data[0];
  ${wgsl_main}
}
`;

  const pipeline = t.device.createComputePipeline({
    layout: 'auto',
    compute: {
      module: t.device.createShaderModule({ code: wgsl }),
      entryPoint: 'main'
    }
  });

  const maxOutputValues = 1000;
  const outputBuffer = t.createBufferTracked({
    size: 4 * (1 + maxOutputValues),
    usage: GPUBufferUsage.STORAGE | GPUBufferUsage.COPY_SRC
  });

  const bindGroup = t.device.createBindGroup({
    layout: pipeline.getBindGroupLayout(0),
    entries: [{ binding: 1, resource: { buffer: outputBuffer } }]
  });

  // Run the shader.
  const encoder = t.device.createCommandEncoder();
  const pass = encoder.beginComputePass();
  pass.setPipeline(pipeline);
  pass.setBindGroup(0, bindGroup);
  pass.dispatchWorkgroups(1);
  pass.end();
  t.queue.submit([encoder.finish()]);

  t.expectGPUBufferValuesEqual(outputBuffer, values);
}

// In each case, the program will write non-zero values to the prefix
// of the output buffer.  The values to check against will cover all
// those values, plus a 0 just beyond the last written value.
const kTests = {
  literal: {
    src: `_ = true;`,
    values: [0]
  },
  call: {
    // RHS evaluated once.
    src: `_ = put(42i);`,
    values: [42, 0]
  },
  call_in_subexpr: {
    src: `_ = put(42i) + 1;`,
    values: [42, 0]
  },
  nested_call: {
    src: `_ = put(put(42)+1);`,
    values: [42, 43, 0]
  },
  unreached: {
    src: `if (false) { _ = put(1); }`,
    values: [0]
  },
  loop: {
    src: `for (var i=5; i<7; i++) { _ = put(i); }`,
    values: [5, 6, 0]
  }
};

g.test('executes').
desc('Tests the RHS is evaluated once when the statement is executed.').
params((u) => u.combine('case', keysOf(kTests))).
fn((t) => {
  runStatementTest(
    t,
    'i32',
    new Int32Array(kTests[t.params.case].values),
    kTests[t.params.case].src
  );
});