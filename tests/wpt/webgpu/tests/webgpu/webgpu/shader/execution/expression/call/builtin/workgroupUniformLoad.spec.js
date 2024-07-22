/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Executes a control barrier synchronization function that affects memory and atomic operations in the workgroup address space.
`; // NOTE: The control barrier executed by this builtin is tested in the memory_model tests.

import { makeTestGroup } from '../../../../../../common/framework/test_group.js';
import { keysOf } from '../../../../../../common/util/data_tables.js';
import {


  iterRange } from
'../../../../../../common/util/util.js';
import { GPUTest } from '../../../../../gpu_test.js';
import { checkElementsEqualGenerated } from '../../../../../util/check_contents.js';

export const g = makeTestGroup(GPUTest);














// A list of types configurations used for the workgroup variable.
const kTypes = {
  bool: {
    store_val: `true`,
    expected: new Uint32Array([1]),
    host_type: 'u32',
    to_host: (x) => `u32(${x})`
  },
  u32: {
    store_val: `42`,
    expected: new Uint32Array([42])
  },
  vec4u: {
    store_val: `vec4u(42, 1, 0xffffffff, 777)`,
    expected: new Uint32Array([42, 1, 0xffffffff, 777])
  },
  mat3x2f: {
    store_val: `mat3x2(42, 1, 65536, -42, -1, -65536)`,
    expected: new Float32Array([42, 1, 65536, -42, -1, -65536])
  },
  'array<u32, 4>': {
    store_val: `array(42, 1, 0xffffffff, 777)`,
    expected: new Uint32Array([42, 1, 0xffffffff, 777])
  },
  SimpleStruct: {
    decls: 'struct SimpleStruct { a: u32, b: u32, c: u32, d: u32, }',
    store_val: `SimpleStruct(42, 1, 0xffffffff, 777)`,
    expected: new Uint32Array([42, 1, 0xffffffff, 777])
  },
  ComplexStruct: {
    decls: `struct Inner { v: vec4u, }
            struct ComplexStruct {
              a: array<Inner, 4>,
              @size(28) b: vec4u,
              c: u32
            }
            const v = vec4(42, 1, 0xffffffff, 777);
            const rhs = ComplexStruct(
              array(Inner(v.xyzw), Inner(v.yzwx), Inner(v.zwxy), Inner(v.wxyz)),
              v.xzxz,
              0x12345678,
              );`,
    store_val: `rhs`,
    expected: new Uint32Array([
    // v.xyzw
    42, 1, 0xffffffff, 777,
    // v.yzwx
    1, 0xffffffff, 777, 42,
    // v.zwxy
    0xffffffff, 777, 42, 1,
    // v.wxyz
    777, 42, 1, 0xffffffff,
    // v.xzxz
    42, 0xffffffff, 42, 0xffffffff,
    // 12 bytes of padding
    0xdeadbeef, 0xdeadbeef, 0xdeadbeef, 0x12345678]
    )
  }
};

g.test('types').
specURL('https://gpuweb.github.io/gpuweb/wgsl/#workgroupUniformLoad-builtin').
desc(
  `Test that the result of a workgroupUniformLoad is the value previously stored to the workgroup variable, for a variety of types.
    `
).
params((u) =>
u.combine('type', keysOf(kTypes)).combine('wgsize', [
[1, 1],
[3, 7],
[1, 128],
[16, 16]]
)
).
fn((t) => {
  const type = kTypes[t.params.type];
  const wgsize_x = t.params.wgsize[0];
  const wgsize_y = t.params.wgsize[1];
  const num_invocations = wgsize_x * wgsize_y;
  const num_words_per_invocation = type.expected.length;
  const total_host_words = num_invocations * num_words_per_invocation;

  t.skipIf(
    num_invocations > t.device.limits.maxComputeInvocationsPerWorkgroup,
    `num_invocations (${num_invocations}) > maxComputeInvocationsPerWorkgroup (${t.device.limits.maxComputeInvocationsPerWorkgroup})`
  );

  let load = `workgroupUniformLoad(&wgvar)`;
  if (type.to_host) {
    load = type.to_host(load);
  }

  // Construct a shader that stores a value to workgroup variable and then loads it using
  // workgroupUniformLoad() in every invocation, copying the results back to a storage buffer.
  const code = `
    ${type.decls ? type.decls : ''}

    @group(0) @binding(0) var<storage, read_write> buffer : array<${
  type.host_type ? type.host_type : t.params.type
  }, ${num_invocations}>;

    var<workgroup> wgvar : ${t.params.type};

    @compute @workgroup_size(${wgsize_x}, ${wgsize_y})
    fn main(@builtin(local_invocation_index) lid: u32) {
      if (lid == ${num_invocations - 1}) {
        wgvar = ${type.store_val};
      }
      buffer[lid] = ${load};
    }
    `;
  const pipeline = t.device.createComputePipeline({
    layout: 'auto',
    compute: {
      module: t.device.createShaderModule({ code }),
      entryPoint: 'main'
    }
  });

  // Allocate a buffer and fill it with 0xdeadbeef values.
  const outputBuffer = t.makeBufferWithContents(
    new Uint32Array([...iterRange(total_host_words, (_i) => 0xdeadbeef)]),
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

  // Check that the output matches the expected values for each invocation.
  t.expectGPUBufferValuesPassCheck(
    outputBuffer,
    (data) =>
    checkElementsEqualGenerated(data, (i) => {
      return Number(type.expected[i % num_words_per_invocation]);
    }),
    {
      type: type.expected.constructor,
      typedLength: total_host_words
    }
  );
});