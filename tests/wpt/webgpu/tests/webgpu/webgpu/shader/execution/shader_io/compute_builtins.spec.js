/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/ export const description = `Test compute shader builtin variables`;
import { makeTestGroup } from '../../../../common/framework/test_group.js';
import { iterRange } from '../../../../common/util/util.js';
import { GPUTest } from '../../../gpu_test.js';

export const g = makeTestGroup(GPUTest);

// Test that the values for each input builtin are correct.
g.test('inputs')
  .desc(`Test compute shader builtin inputs values`)
  .params(u =>
    u
      .combine('method', ['param', 'struct', 'mixed'])
      .combine('dispatch', ['direct', 'indirect'])
      .combineWithParams([
        {
          groupSize: { x: 1, y: 1, z: 1 },
          numGroups: { x: 1, y: 1, z: 1 },
        },
        {
          groupSize: { x: 8, y: 4, z: 2 },
          numGroups: { x: 1, y: 1, z: 1 },
        },
        {
          groupSize: { x: 1, y: 1, z: 1 },
          numGroups: { x: 8, y: 4, z: 2 },
        },
        {
          groupSize: { x: 3, y: 7, z: 5 },
          numGroups: { x: 13, y: 9, z: 11 },
        },
      ])
      .beginSubcases()
  )
  .fn(t => {
    const invocationsPerGroup = t.params.groupSize.x * t.params.groupSize.y * t.params.groupSize.z;
    const totalInvocations =
      invocationsPerGroup * t.params.numGroups.x * t.params.numGroups.y * t.params.numGroups.z;

    // Generate the structures, parameters, and builtin expressions used in the shader.
    let params = '';
    let structures = '';
    let local_id = '';
    let local_index = '';
    let global_id = '';
    let group_id = '';
    let num_groups = '';
    switch (t.params.method) {
      case 'param':
        params = `
          @builtin(local_invocation_id) local_id : vec3<u32>,
          @builtin(local_invocation_index) local_index : u32,
          @builtin(global_invocation_id) global_id : vec3<u32>,
          @builtin(workgroup_id) group_id : vec3<u32>,
          @builtin(num_workgroups) num_groups : vec3<u32>,
        `;
        local_id = 'local_id';
        local_index = 'local_index';
        global_id = 'global_id';
        group_id = 'group_id';
        num_groups = 'num_groups';
        break;
      case 'struct':
        structures = `struct Inputs {
            @builtin(local_invocation_id) local_id : vec3<u32>,
            @builtin(local_invocation_index) local_index : u32,
            @builtin(global_invocation_id) global_id : vec3<u32>,
            @builtin(workgroup_id) group_id : vec3<u32>,
            @builtin(num_workgroups) num_groups : vec3<u32>,
          };`;
        params = `inputs : Inputs`;
        local_id = 'inputs.local_id';
        local_index = 'inputs.local_index';
        global_id = 'inputs.global_id';
        group_id = 'inputs.group_id';
        num_groups = 'inputs.num_groups';
        break;
      case 'mixed':
        structures = `struct InputsA {
          @builtin(local_invocation_index) local_index : u32,
          @builtin(global_invocation_id) global_id : vec3<u32>,
        };
        struct InputsB {
          @builtin(workgroup_id) group_id : vec3<u32>
        };`;
        params = `@builtin(local_invocation_id) local_id : vec3<u32>,
                  inputsA : InputsA,
                  inputsB : InputsB,
                  @builtin(num_workgroups) num_groups : vec3<u32>,`;
        local_id = 'local_id';
        local_index = 'inputsA.local_index';
        global_id = 'inputsA.global_id';
        group_id = 'inputsB.group_id';
        num_groups = 'num_groups';
        break;
    }

    // WGSL shader that stores every builtin value to a buffer, for every invocation in the grid.
    const wgsl = `
      struct S {
        data : array<u32>
      };
      struct V {
        data : array<vec3<u32>>
      };
      @group(0) @binding(0) var<storage, read_write> local_id_out : V;
      @group(0) @binding(1) var<storage, read_write> local_index_out : S;
      @group(0) @binding(2) var<storage, read_write> global_id_out : V;
      @group(0) @binding(3) var<storage, read_write> group_id_out : V;
      @group(0) @binding(4) var<storage, read_write> num_groups_out : V;

      ${structures}

      const group_width = ${t.params.groupSize.x}u;
      const group_height = ${t.params.groupSize.y}u;
      const group_depth = ${t.params.groupSize.z}u;

      @compute @workgroup_size(group_width, group_height, group_depth)
      fn main(
        ${params}
        ) {
        let group_index = ((${group_id}.z * ${num_groups}.y) + ${group_id}.y) * ${num_groups}.x + ${group_id}.x;
        let global_index = group_index * ${invocationsPerGroup}u + ${local_index};
        local_id_out.data[global_index] = ${local_id};
        local_index_out.data[global_index] = ${local_index};
        global_id_out.data[global_index] = ${global_id};
        group_id_out.data[global_index] = ${group_id};
        num_groups_out.data[global_index] = ${num_groups};
      }
    `;

    const pipeline = t.device.createComputePipeline({
      layout: 'auto',
      compute: {
        module: t.device.createShaderModule({
          code: wgsl,
        }),
        entryPoint: 'main',
      },
    });

    // Helper to create a `size`-byte buffer with binding number `binding`.
    function createBuffer(size, binding) {
      const buffer = t.device.createBuffer({
        size,
        usage: GPUBufferUsage.STORAGE | GPUBufferUsage.COPY_SRC,
      });
      t.trackForCleanup(buffer);

      bindGroupEntries.push({
        binding,
        resource: {
          buffer,
        },
      });

      return buffer;
    }

    // Create the output buffers.
    const bindGroupEntries = [];
    const localIdBuffer = createBuffer(totalInvocations * 16, 0);
    const localIndexBuffer = createBuffer(totalInvocations * 4, 1);
    const globalIdBuffer = createBuffer(totalInvocations * 16, 2);
    const groupIdBuffer = createBuffer(totalInvocations * 16, 3);
    const numGroupsBuffer = createBuffer(totalInvocations * 16, 4);

    const bindGroup = t.device.createBindGroup({
      layout: pipeline.getBindGroupLayout(0),
      entries: bindGroupEntries,
    });

    // Run the shader.
    const encoder = t.device.createCommandEncoder();
    const pass = encoder.beginComputePass();
    pass.setPipeline(pipeline);
    pass.setBindGroup(0, bindGroup);
    switch (t.params.dispatch) {
      case 'direct':
        pass.dispatchWorkgroups(t.params.numGroups.x, t.params.numGroups.y, t.params.numGroups.z);
        break;
      case 'indirect': {
        const dispatchBuffer = t.device.createBuffer({
          size: 3 * Uint32Array.BYTES_PER_ELEMENT,
          usage: GPUBufferUsage.INDIRECT,
          mappedAtCreation: true,
        });
        t.trackForCleanup(dispatchBuffer);
        const dispatchData = new Uint32Array(dispatchBuffer.getMappedRange());
        dispatchData[0] = t.params.numGroups.x;
        dispatchData[1] = t.params.numGroups.y;
        dispatchData[2] = t.params.numGroups.z;
        dispatchBuffer.unmap();
        pass.dispatchWorkgroupsIndirect(dispatchBuffer, 0);
        break;
      }
    }

    pass.end();
    t.queue.submit([encoder.finish()]);

    // Helper to check that the vec3<u32> value at each index of the provided `output` buffer
    // matches the expected value for that invocation, as generated by the `getBuiltinValue`
    // function. The `name` parameter is the builtin name, used for error messages.
    const checkEachIndex = (output, name, getBuiltinValue) => {
      // Loop over workgroups.
      for (let gz = 0; gz < t.params.numGroups.z; gz++) {
        for (let gy = 0; gy < t.params.numGroups.y; gy++) {
          for (let gx = 0; gx < t.params.numGroups.x; gx++) {
            // Loop over invocations within a group.
            for (let lz = 0; lz < t.params.groupSize.z; lz++) {
              for (let ly = 0; ly < t.params.groupSize.y; ly++) {
                for (let lx = 0; lx < t.params.groupSize.x; lx++) {
                  const groupIndex = (gz * t.params.numGroups.y + gy) * t.params.numGroups.x + gx;
                  const localIndex = (lz * t.params.groupSize.y + ly) * t.params.groupSize.x + lx;
                  const globalIndex = groupIndex * invocationsPerGroup + localIndex;
                  const expected = getBuiltinValue(
                    { x: gx, y: gy, z: gz },
                    { x: lx, y: ly, z: lz }
                  );

                  if (output[globalIndex * 4 + 0] !== expected.x) {
                    return new Error(
                      `${name}.x failed at group(${gx},${gy},${gz}) local(${lx},${ly},${lz}))\n` +
                        `    expected: ${expected.x}\n` +
                        `    got:      ${output[globalIndex * 4 + 0]}`
                    );
                  }
                  if (output[globalIndex * 4 + 1] !== expected.y) {
                    return new Error(
                      `${name}.y failed at group(${gx},${gy},${gz}) local(${lx},${ly},${lz}))\n` +
                        `    expected: ${expected.y}\n` +
                        `    got:      ${output[globalIndex * 4 + 1]}`
                    );
                  }
                  if (output[globalIndex * 4 + 2] !== expected.z) {
                    return new Error(
                      `${name}.z failed at group(${gx},${gy},${gz}) local(${lx},${ly},${lz}))\n` +
                        `    expected: ${expected.z}\n` +
                        `    got:      ${output[globalIndex * 4 + 2]}`
                    );
                  }
                }
              }
            }
          }
        }
      }
      return undefined;
    };

    // Check @builtin(local_invocation_index) values.
    t.expectGPUBufferValuesEqual(
      localIndexBuffer,
      new Uint32Array([...iterRange(totalInvocations, x => x % invocationsPerGroup)])
    );

    // Check @builtin(local_invocation_id) values.
    t.expectGPUBufferValuesPassCheck(
      localIdBuffer,
      outputData => checkEachIndex(outputData, 'local_invocation_id', (_, localId) => localId),
      { type: Uint32Array, typedLength: totalInvocations * 4 }
    );

    // Check @builtin(global_invocation_id) values.
    const getGlobalId = (groupId, localId) => {
      return {
        x: groupId.x * t.params.groupSize.x + localId.x,
        y: groupId.y * t.params.groupSize.y + localId.y,
        z: groupId.z * t.params.groupSize.z + localId.z,
      };
    };
    t.expectGPUBufferValuesPassCheck(
      globalIdBuffer,
      outputData => checkEachIndex(outputData, 'global_invocation_id', getGlobalId),
      { type: Uint32Array, typedLength: totalInvocations * 4 }
    );

    // Check @builtin(workgroup_id) values.
    t.expectGPUBufferValuesPassCheck(
      groupIdBuffer,
      outputData => checkEachIndex(outputData, 'workgroup_id', (groupId, _) => groupId),
      { type: Uint32Array, typedLength: totalInvocations * 4 }
    );

    // Check @builtin(num_workgroups) values.
    t.expectGPUBufferValuesPassCheck(
      numGroupsBuffer,
      outputData => checkEachIndex(outputData, 'num_workgroups', () => t.params.numGroups),
      { type: Uint32Array, typedLength: totalInvocations * 4 }
    );
  });
