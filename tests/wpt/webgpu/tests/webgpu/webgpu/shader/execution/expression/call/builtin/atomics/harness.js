/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/import { assert } from

'../../../../../../../common/util/util.js';


// Use these in combination.
export const workgroupSizes = [1, 2, 32, 64];
export const dispatchSizes = [1, 4, 8, 16];

// Use this alone - dispatch size should be 1.
export const onlyWorkgroupSizes = [1, 2, 4, 8, 16, 32, 64, 128, 256];

export const kMapId = {
  passthrough: {
    f: (id, _max) => id,
    wgsl: (_max, scalarType = 'u32') =>
    `fn map_id(id: ${scalarType}) -> ${scalarType} { return id; }`
  },
  remap: {
    f: (id, max) => ((id >>> 0) * 14957 ^ (id >>> 0) * 26561 >> 2) % max,
    wgsl: (max, scalarType = 'u32') =>
    `fn map_id(id: ${scalarType}) -> ${scalarType} { return ((id * 14957) ^ ((id * 26561) >> 2)) % ${max}; }`
  }
};

export function typedArrayCtor(scalarType) {
  switch (scalarType) {
    case 'u32':
      return Uint32Array;
    case 'i32':
      return Int32Array;
    default:
      assert(false, 'Atomic variables can only by u32 or i32');
      return Uint8Array;
  }
}

export function runStorageVariableTest({
  t,
  workgroupSize, // Workgroup X-size
  dispatchSize, // Dispatch X-size
  bufferNumElements, // Number of 32-bit elements in output buffer
  initValue, // 32-bit initial value used to fill output buffer
  // Atomic op source executed by the compute shader, NOTE: 'id' is global_invocation_id.x,
  // and `output` is a storage array of atomics.
  op,
  expected, // Expected values array to compare against output buffer
  extra // Optional extra WGSL source









}) {
  assert(expected.length === bufferNumElements, "'expected' buffer size is incorrect");

  const scalarType = expected instanceof Uint32Array ? 'u32' : 'i32';
  const arrayType = typedArrayCtor(scalarType);

  const wgsl = `
    @group(0) @binding(0)
    var<storage, read_write> output : array<atomic<${scalarType}>>;

    @compute @workgroup_size(${workgroupSize})
    fn main(
        @builtin(global_invocation_id) global_invocation_id : vec3<u32>,
        ) {
      let id = ${scalarType}(global_invocation_id[0]);
      ${op};
    }
    ${extra || ''}
    `;

  const pipeline = t.device.createComputePipeline({
    layout: 'auto',
    compute: {
      module: t.device.createShaderModule({ code: wgsl }),
      entryPoint: 'main'
    }
  });

  const outputBuffer = t.device.createBuffer({
    size: bufferNumElements * arrayType.BYTES_PER_ELEMENT,
    usage: GPUBufferUsage.STORAGE | GPUBufferUsage.COPY_SRC,
    mappedAtCreation: true
  });
  // Fill with initial value
  t.trackForCleanup(outputBuffer);
  const data = new arrayType(outputBuffer.getMappedRange());
  data.fill(initValue);
  outputBuffer.unmap();

  const bindGroup = t.device.createBindGroup({
    layout: pipeline.getBindGroupLayout(0),
    entries: [{ binding: 0, resource: { buffer: outputBuffer } }]
  });

  // Run the shader.
  const encoder = t.device.createCommandEncoder();
  const pass = encoder.beginComputePass();
  pass.setPipeline(pipeline);
  pass.setBindGroup(0, bindGroup);
  pass.dispatchWorkgroups(dispatchSize);
  pass.end();
  t.queue.submit([encoder.finish()]);

  t.expectGPUBufferValuesEqual(outputBuffer, expected);
}

export function runWorkgroupVariableTest({
  t,
  workgroupSize, // Workgroup X-size
  dispatchSize, // Dispatch X-size
  wgNumElements, // Number of 32-bit elements in 'wg' array. Output buffer is sized to wgNumElements * dispatchSize.
  initValue, // 32-bit initial value used to fill 'wg' array
  // Atomic op source executed by the compute shader, NOTE: 'id' is local_invocation_index,
  // `wg` is a workgroup array of atomics of size `workgroupSize`, `output` is a storage array of non-atomics of size
  // `workgroupSize * dispatcSize` to which each dispatch of `wg` gets copied to (dispatch 0 to first workgroupSize elements,
  // dispatch 1 to second workgroupSize elements, etc.).
  op,
  expected, // Expected values array to compare against output buffer
  extra // Optional extra WGSL source









}) {
  assert(expected.length === wgNumElements * dispatchSize, "'expected' buffer size is incorrect");

  const scalarType = expected instanceof Uint32Array ? 'u32' : 'i32';
  const arrayType = typedArrayCtor(scalarType);

  const wgsl = `
    var<workgroup> wg: array<atomic<${scalarType}>, ${wgNumElements}>;

    // Result of each workgroup is written to output[workgroup_id.x]
    @group(0) @binding(0)
    var<storage, read_write> output: array<${scalarType}, ${wgNumElements * dispatchSize}>;

    @compute @workgroup_size(${workgroupSize})
    fn main(
        @builtin(local_invocation_index) local_invocation_index: u32,
        @builtin(workgroup_id) workgroup_id : vec3<u32>
        ) {
      let id = ${scalarType}(local_invocation_index);
      let global_id = ${scalarType}(workgroup_id.x * ${wgNumElements} + local_invocation_index);

      // Initialize workgroup array
      if (local_invocation_index == 0) {
        for (var i = 0u; i < ${wgNumElements}; i++) {
          atomicStore(&wg[i], bitcast<${scalarType}>(${initValue}u));
        }
      }
      workgroupBarrier();

      ${op};

      // Copy results to output buffer
      workgroupBarrier();
      if (local_invocation_index == 0) {
        for (var i = 0u; i < ${wgNumElements}; i++) {
          output[(workgroup_id.x * ${wgNumElements}) + i] = atomicLoad(&wg[i]);
        }
      }
    }
    ${extra || ''}
    `;

  const pipeline = t.device.createComputePipeline({
    layout: 'auto',
    compute: {
      module: t.device.createShaderModule({ code: wgsl }),
      entryPoint: 'main'
    }
  });

  const outputBuffer = t.device.createBuffer({
    size: wgNumElements * dispatchSize * arrayType.BYTES_PER_ELEMENT,
    usage: GPUBufferUsage.STORAGE | GPUBufferUsage.COPY_SRC
  });

  const bindGroup = t.device.createBindGroup({
    layout: pipeline.getBindGroupLayout(0),
    entries: [{ binding: 0, resource: { buffer: outputBuffer } }]
  });

  // Run the shader.
  const encoder = t.device.createCommandEncoder();
  const pass = encoder.beginComputePass();
  pass.setPipeline(pipeline);
  pass.setBindGroup(0, bindGroup);
  pass.dispatchWorkgroups(dispatchSize);
  pass.end();
  t.queue.submit([encoder.finish()]);

  t.expectGPUBufferValuesEqual(outputBuffer, expected);
}