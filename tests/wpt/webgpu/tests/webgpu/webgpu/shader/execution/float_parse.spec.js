/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Execution Tests for float parsing cases
`;import { makeTestGroup } from '../../../common/framework/test_group.js';
import { keysOf } from '../../../common/util/data_tables.js';
import { iterRange } from '../../../common/util/util.js';
import { GPUTest } from '../../gpu_test.js';

export const g = makeTestGroup(GPUTest);

/**
 * Run a shader and check that the buffer output matches expectations.
 *
 * @param t The test object
 * @param wgsl The shader source
 * @param expected The array of expected values after running the shader
 */
function runShaderTest(t, wgsl, expected) {
  const pipeline = t.device.createComputePipeline({
    layout: 'auto',
    compute: {
      module: t.device.createShaderModule({ code: wgsl }),
      entryPoint: 'main'
    }
  });

  // Allocate a buffer and fill it with 0xdeadbeef words.
  const outputBuffer = t.makeBufferWithContents(
    new Float32Array([...iterRange(expected.length, (_i) => 0xdeadbeef)]),
    GPUBufferUsage.STORAGE | GPUBufferUsage.COPY_SRC
  );
  const bindGroup = t.device.createBindGroup({
    layout: pipeline.getBindGroupLayout(0),
    entries: [{ binding: 0, resource: { buffer: outputBuffer } }]
  });

  // Run the shader.
  const encoder = t.device.createCommandEncoder();
  const pass = encoder.beginComputePass();
  pass.setPipeline(pipeline);
  pass.setBindGroup(0, bindGroup);
  pass.dispatchWorkgroups(1);
  pass.end();
  t.queue.submit([encoder.finish()]);

  // Check that only the non-padding bytes were modified.
  t.expectGPUBufferValuesEqual(outputBuffer, expected);
}

const kTestFloats = {
  small_pos_zero_exp: {
    src:
    '0.' +
    '00000000000000000000000000000000000000000000000000' + //  50
    '00000000000000000000000000000000000000000000000000' + // 100
    '00000000000000000000000000000000000000000000000000' + // 150
    '00000000000000000000000000000000000000000000000000' + // 200
    '00000000000000000000000000000000000000000000000000' + // 250
    '00000000000000000000000000000000000000000000000000' + // 300
    '00000000000000000000000000000000000000000000000000' + // 350
    '1e+0',
    result: 0.0
  },
  small_pos_non_zero_exp: {
    src:
    '0.' +
    '00000000000000000000000000000000000000000000000000' + //  50
    '00000000000000000000000000000000000000000000000000' + // 100
    '00000000000000000000000000000000000000000000000000' + // 150
    '00000000000000000000000000000000000000000000000000' + // 200
    '00000000000000000000000000000000000000000000000000' + // 250
    '00000000000000000000000000000000000000000000000000' + // 300
    '00000000000000000000000000000000000000000000000000' + // 350
    '1e+10',
    result: 0.0
  },
  pos_exp_neg_result: {
    src:
    '0.' +
    '00000000000000000000000000000000000000000000000000' + //  50
    '00000000000000000000000000000000000000000000000000' + // 100
    '00000000000000000000000000000000000000000000000000' + // 150
    '00000000000000000000000000000000000000000000000000' + // 200
    '00000000000000000000000000000000000000000000000000' + // 250
    '00000000000000000000000000000000000000000000000000' + // 300
    '00000000000000000000000000000000000000000000000000' + // 350
    '1e+300',
    result: 1e-51
  },
  no_exp: {
    src:
    '0.' +
    '00000000000000000000000000000000000000000000000000' + //  50
    '00000000000000000000000000000000000000000000000000' + // 100
    '00000000000000000000000000000000000000000000000000' + // 150
    '00000000000000000000000000000000000000000000000000' + // 200
    '00000000000000000000000000000000000000000000000000' + // 250
    '00000000000000000000000000000000000000000000000000' + // 300
    '00000000000000000000000000000000000000000000000000' + // 350
    '1',
    result: 0.0
  },
  large_number_small_exp: {
    src:
    '1' +
    '00000000000000000000000000000000000000000000000000' + //  50
    '00000000000000000000000000000000000000000000000000' + // 100
    '.0e-350',
    result: 1e-251
  }
};

g.test('valid').
desc(`Test that floats are parsed correctly`).
params((u) => u.combine('value', keysOf(kTestFloats))).
fn((t) => {
  const data = kTestFloats[t.params.value];
  const wgsl = `
      struct S {
        val: f32,
      }
      @group(0) @binding(0) var<storage, read_write> buffer : S;

      @compute @workgroup_size(1)
      fn main() {
        buffer = S(${data.src});
      }
    `;
  runShaderTest(t, wgsl, new Float32Array([data.result]));
});