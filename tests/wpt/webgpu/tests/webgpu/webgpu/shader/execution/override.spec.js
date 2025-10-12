/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `Test override execution`;import { makeTestGroup } from '../../../common/framework/test_group.js';
import { keysOf } from '../../../common/util/data_tables.js';
import { AllFeaturesMaxLimitsGPUTest } from '../../gpu_test.js';
import { checkElementsEqual } from '../../util/check_contents.js';

export const g = makeTestGroup(AllFeaturesMaxLimitsGPUTest);

const kOverrideCases = {
  logical_lhs_override: {
    code: `x == 2 && 1 < 2`
  },
  logical_rhs_override: {
    code: `1 > 2 || x == 2`
  },
  logical_both_override: {
    code: `x > 2 || x == 2`
  }
};

g.test('logical').
desc(`Test replacing an override in the LHS of a logical statement`).
params((u) => u.combine('case', keysOf(kOverrideCases))).
fn(async (t) => {
  const expr = kOverrideCases[t.params.case].code;
  const code = `
override x: u32 = 2;
override y: bool = ${expr};

@group(0) @binding(0) var<storage, read_write> v : vec4u;

@compute @workgroup_size(1)
fn main() {
  if (y) {
      v = vec4u(4, 4, 4, 4);
  } else {
      v = vec4u(1, 1, 1, 1);
  }
}`;

  const pipeline = t.device.createComputePipeline({
    layout: 'auto',
    compute: {
      module: t.device.createShaderModule({
        code
      }),
      entryPoint: 'main'
    }
  });

  const buffer = t.makeBufferWithContents(
    new Uint32Array([0, 0, 0, 0]),
    GPUBufferUsage.STORAGE | GPUBufferUsage.COPY_SRC | GPUBufferUsage.COPY_DST
  );

  const bg = t.device.createBindGroup({
    layout: pipeline.getBindGroupLayout(0),
    entries: [
    {
      binding: 0,
      resource: {
        buffer
      }
    }]

  });

  const encoder = t.device.createCommandEncoder();
  const pass = encoder.beginComputePass();
  pass.setPipeline(pipeline);
  pass.setBindGroup(0, bg);
  pass.dispatchWorkgroups(1, 1, 1);
  pass.end();
  t.queue.submit([encoder.finish()]);

  const bufferReadback = await t.readGPUBufferRangeTyped(buffer, {
    srcByteOffset: 0,
    type: Uint32Array,
    typedLength: 4,
    method: 'copy'
  });
  const got = bufferReadback.data;
  const expected = new Uint32Array([4, 4, 4, 4]);

  t.expectOK(checkElementsEqual(got, expected));
});