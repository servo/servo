/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Compound statement execution.
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

fn put(value : ${ty}) {
  outputs.data[count] = value;
  count += 1;
}

@compute @workgroup_size(1)
fn main() {
  _ = &outputs;
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
  const outputBuffer = t.device.createBuffer({
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

// Consider a declaration X of identifier 'x' inside a compound statement.
// Check the value of 'x' at various places relative to X:
//     a { b; X=c; d; { e; } } f;

const kTests = {
  uses: {
    // Observe values without conflicting declarations.
    src: `let x = 1;
          put(x);
          {
            put(x);
            let x = x+1;  // The declaration in question
            put(x);
            {
              put(x);
            }
            put(x);
          }
          put(x);`,
    values: [1, 1, 2, 2, 2, 1]
  },
  shadowed: {
    // Observe values when shadowed
    src: `let x = 1;
          put(x);
          {
            put(x);
            let x = x+1;  // The declaration in question
            put(x);
            {
              let x = x+1;  // A shadow
              put(x);
            }
            put(x);
          }
          put(x);`,
    values: [1, 1, 2, 3, 2, 1]
  },
  gone: {
    // The declaration goes out of scope.
    src: `{
            let x = 2;  // The declaration in question
            put(x);
          }
          let x = 1;
          put(x);`,
    values: [2, 1]
  }
};

g.test('decl').
desc('Tests the value of a declared value in a compound statment.').
params((u) => u.combine('case', keysOf(kTests))).
fn((t) => {
  runStatementTest(
    t,
    'i32',
    new Int32Array(kTests[t.params.case].values),
    kTests[t.params.case].src
  );
});