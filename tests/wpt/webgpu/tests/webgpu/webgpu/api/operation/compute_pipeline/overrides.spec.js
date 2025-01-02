/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Compute pipeline using overridable constants test.
`;import { makeTestGroup } from '../../../../common/framework/test_group.js';
import { range } from '../../../../common/util/util.js';
import { GPUTest } from '../../../gpu_test.js';

class F extends GPUTest {
  async ExpectShaderOutputWithConstants(
  isAsync,
  expected,
  constants,
  code)
  {
    const dst = this.createBufferTracked({
      size: expected.byteLength,
      usage: GPUBufferUsage.COPY_SRC | GPUBufferUsage.STORAGE
    });

    const descriptor = {
      layout: 'auto',
      compute: {
        module: this.device.createShaderModule({
          code
        }),
        entryPoint: 'main',
        constants
      }
    };

    const promise = isAsync ?
    this.device.createComputePipelineAsync(descriptor) :
    Promise.resolve(this.device.createComputePipeline(descriptor));

    const pipeline = await promise;
    const bindGroup = this.device.createBindGroup({
      entries: [{ binding: 0, resource: { buffer: dst, offset: 0, size: expected.byteLength } }],
      layout: pipeline.getBindGroupLayout(0)
    });

    const encoder = this.device.createCommandEncoder();
    const pass = encoder.beginComputePass();
    pass.setPipeline(pipeline);
    pass.setBindGroup(0, bindGroup);
    pass.dispatchWorkgroups(1);
    pass.end();
    this.device.queue.submit([encoder.finish()]);

    this.expectGPUBufferValuesEqual(dst, expected);
  }
}

export const g = makeTestGroup(F);

g.test('basic').
desc(
  `Test that either correct constants override values or default values when no constants override value are provided at pipeline creation time are used as the output to the storage buffer.`
).
params((u) => u.combine('isAsync', [true, false])).
fn(async (t) => {
  const count = 11;
  await t.ExpectShaderOutputWithConstants(
    t.params.isAsync,
    new Uint32Array(range(count, (i) => i)),
    {
      c0: 0,
      c1: 1,
      c2: 2,
      c3: 3,
      // c4 is using default value
      c5: 5,
      c6: 6,
      // c7 is using default value
      c8: 8,
      c9: 9
      // c10 is using default value
    },
    `
      override c0: bool;              // type: bool
      override c1: bool = false;      // default override
      override c2: f32;               // type: float32
      override c3: f32 = 0.0;         // default override
      override c4: f32 = 4.0;         // default
      override c5: i32;               // type: int32
      override c6: i32 = 0;           // default override
      override c7: i32 = 7;           // default
      override c8: u32;               // type: uint32
      override c9: u32 = 0u;          // default override
      override c10: u32 = 10u;        // default

      struct Buf {
          data : array<u32, ${count}>
      }

      @group(0) @binding(0) var<storage, read_write> buf : Buf;

      @compute @workgroup_size(1) fn main() {
          buf.data[0] = u32(c0);
          buf.data[1] = u32(c1);
          buf.data[2] = u32(c2);
          buf.data[3] = u32(c3);
          buf.data[4] = u32(c4);
          buf.data[5] = u32(c5);
          buf.data[6] = u32(c6);
          buf.data[7] = u32(c7);
          buf.data[8] = u32(c8);
          buf.data[9] = u32(c9);
          buf.data[10] = u32(c10);
      }
    `
  );
});

g.test('numeric_id').
desc(
  `Test that correct values are used as output to the storage buffer for constants specified with numeric id instead of their names.`
).
params((u) => u.combine('isAsync', [true, false])).
fn(async (t) => {
  await t.ExpectShaderOutputWithConstants(
    t.params.isAsync,
    new Uint32Array([1, 2, 3]),
    {
      1001: 1,
      1: 2
      // 1003 is using default value
    },
    `
        @id(1001) override c1: u32;            // some big numeric id
        @id(1) override c2: u32 = 0u;          // id == 1 might collide with some generated constant id
        @id(1003) override c3: u32 = 3u;       // default

        struct Buf {
            data : array<u32, 3>
        }

        @group(0) @binding(0) var<storage, read_write> buf : Buf;

        @compute @workgroup_size(1) fn main() {
            buf.data[0] = c1;
            buf.data[1] = c2;
            buf.data[2] = c3;
        }
      `
  );
});

g.test('computed').
desc(`Test that computed overrides work correctly`).
fn(async (t) => {
  const module = t.device.createShaderModule({
    code: `
      override c0: f32 = 0.;
      override c1: f32 = 0.;
      override c2: f32 = c0 * c1;

      struct Buf {
          data : array<u32, 3>,
      }

      @group(0) @binding(0) var<storage, read_write> buf : Buf;

      @compute @workgroup_size(1) fn main() {
          buf.data[0] = u32(c0);
          buf.data[1] = u32(c1);
          buf.data[2] = u32(c2);
      }
    `
  });

  const expected = new Uint32Array([2, 4, 8]);

  const buffer = t.createBufferTracked({
    size: 3 * Uint32Array.BYTES_PER_ELEMENT,
    usage: GPUBufferUsage.COPY_SRC | GPUBufferUsage.STORAGE
  });

  const descriptors = [
  {
    layout: 'auto',
    compute: {
      module,
      entryPoint: 'main',
      constants: {
        c0: 2,
        c1: 4
      }
    }
  }];


  const pipeline = await t.device.createComputePipelineAsync(descriptors[0]);
  const bindGroups = [
  t.device.createBindGroup({
    entries: [
    {
      binding: 0,
      resource: { buffer, offset: 0, size: 3 * Uint32Array.BYTES_PER_ELEMENT }
    }],

    layout: pipeline.getBindGroupLayout(0)
  })];


  const encoder = t.device.createCommandEncoder();
  const pass = encoder.beginComputePass();
  pass.setPipeline(pipeline);
  pass.setBindGroup(0, bindGroups[0]);
  pass.dispatchWorkgroups(1);
  pass.end();
  t.device.queue.submit([encoder.finish()]);

  t.expectGPUBufferValuesEqual(buffer, expected);
});

g.test('precision').
desc(
  `Test that float number precision is preserved for constants as they are used for compute shader output of the storage buffer.`
).
params((u) => u.combine('isAsync', [true, false])).
fn(async (t) => {
  const c1 = 3.14159;
  const c2 = 3.141592653589793;
  await t.ExpectShaderOutputWithConstants(
    t.params.isAsync,
    // These values will get rounded to f32 and createComputePipeline, so the values coming out from the shader won't be the exact same one as shown here.
    new Float32Array([c1, c2]),
    {
      c1,
      c2
    },
    `
        override c1: f32;
        override c2: f32;

        struct Buf {
            data : array<f32, 2>
        }

        @group(0) @binding(0) var<storage, read_write> buf : Buf;

        @compute @workgroup_size(1) fn main() {
            buf.data[0] = c1;
            buf.data[1] = c2;
        }
      `
  );
});

g.test('workgroup_size').
desc(
  `Test that constants can be used as workgroup size correctly, the compute shader should write the max local invocation id to the storage buffer which is equal to the workgroup size dimension given by the constant.`
).
params((u) =>
u //
.combine('isAsync', [true, false]).
combine('type', ['u32', 'i32']).
combine('size', [3, 16, 64]).
combine('v', ['x', 'y', 'z'])
).
fn(async (t) => {
  const { isAsync, type, size, v } = t.params;
  const workgroup_size_str = v === 'x' ? 'd' : v === 'y' ? '1, d' : '1, 1, d';
  await t.ExpectShaderOutputWithConstants(
    isAsync,
    new Uint32Array([size]),
    {
      d: size
    },
    `
        override d: ${type};

        struct Buf {
            data : array<u32, 1>
        }

        @group(0) @binding(0) var<storage, read_write> buf : Buf;

        @compute @workgroup_size(${workgroup_size_str}) fn main(
            @builtin(local_invocation_id) local_invocation_id : vec3<u32>
        ) {
            if (local_invocation_id.${v} >= u32(d - 1)) {
                buf.data[0] = local_invocation_id.${v} + 1;
            }
        }
      `
  );
});

g.test('shared_shader_module').
desc(
  `Test that when the same shader module is shared by different pipelines, the correct constant values are used as output to the storage buffer. The constant value should not affect other pipeline sharing the same shader module.`
).
params((u) => u.combine('isAsync', [true, false])).
fn(async (t) => {
  const module = t.device.createShaderModule({
    code: `
      override a: u32;

      struct Buf {
          data : array<u32, 1>
      }

      @group(0) @binding(0) var<storage, read_write> buf : Buf;

      @compute @workgroup_size(1) fn main() {
          buf.data[0] = a;
      }`
  });

  const expects = [new Uint32Array([1]), new Uint32Array([2])];
  const buffers = [
  t.createBufferTracked({
    size: Uint32Array.BYTES_PER_ELEMENT,
    usage: GPUBufferUsage.COPY_SRC | GPUBufferUsage.STORAGE
  }),
  t.createBufferTracked({
    size: Uint32Array.BYTES_PER_ELEMENT,
    usage: GPUBufferUsage.COPY_SRC | GPUBufferUsage.STORAGE
  })];


  const descriptors = [
  {
    layout: 'auto',
    compute: {
      module,
      entryPoint: 'main',
      constants: {
        a: 1
      }
    }
  },
  {
    layout: 'auto',
    compute: {
      module,
      entryPoint: 'main',
      constants: {
        a: 2
      }
    }
  }];


  const promises = t.params.isAsync ?
  Promise.all([
  t.device.createComputePipelineAsync(descriptors[0]),
  t.device.createComputePipelineAsync(descriptors[1])]
  ) :
  Promise.resolve([
  t.device.createComputePipeline(descriptors[0]),
  t.device.createComputePipeline(descriptors[1])]
  );

  const pipelines = await promises;
  const bindGroups = [
  t.device.createBindGroup({
    entries: [
    {
      binding: 0,
      resource: { buffer: buffers[0], offset: 0, size: Uint32Array.BYTES_PER_ELEMENT }
    }],

    layout: pipelines[0].getBindGroupLayout(0)
  }),
  t.device.createBindGroup({
    entries: [
    {
      binding: 0,
      resource: { buffer: buffers[1], offset: 0, size: Uint32Array.BYTES_PER_ELEMENT }
    }],

    layout: pipelines[1].getBindGroupLayout(0)
  })];


  const encoder = t.device.createCommandEncoder();
  const pass = encoder.beginComputePass();
  pass.setPipeline(pipelines[0]);
  pass.setBindGroup(0, bindGroups[0]);
  pass.dispatchWorkgroups(1);
  pass.setPipeline(pipelines[1]);
  pass.setBindGroup(0, bindGroups[1]);
  pass.dispatchWorkgroups(1);
  pass.end();
  t.device.queue.submit([encoder.finish()]);

  t.expectGPUBufferValuesEqual(buffers[0], expects[0]);
  t.expectGPUBufferValuesEqual(buffers[1], expects[1]);
});

g.test('multi_entry_points').
desc(
  `Test that constants used for different entry points are used correctly as output to the storage buffer. They should have no impact for pipeline using entry points that doesn't reference them.`
).
params((u) => u.combine('isAsync', [true, false])).
fn(async (t) => {
  const module = t.device.createShaderModule({
    code: `
    override c1: u32;
    override c2: u32;
    override c3: u32;

    struct Buf {
        data : array<u32, 1>
    }

    @group(0) @binding(0) var<storage, read_write> buf : Buf;

    @compute @workgroup_size(1) fn main1() {
        buf.data[0] = c1;
    }

    @compute @workgroup_size(1) fn main2() {
        buf.data[0] = c2;
    }

    @compute @workgroup_size(c3) fn main3() {
        buf.data[0] = 3u;
    }`
  });

  const expects = [
  new Uint32Array([1]),
  new Uint32Array([2]),
  new Uint32Array([3]),
  new Uint32Array([4])];


  const buffers = [
  t.createBufferTracked({
    size: Uint32Array.BYTES_PER_ELEMENT,
    usage: GPUBufferUsage.COPY_SRC | GPUBufferUsage.STORAGE
  }),
  t.createBufferTracked({
    size: Uint32Array.BYTES_PER_ELEMENT,
    usage: GPUBufferUsage.COPY_SRC | GPUBufferUsage.STORAGE
  }),
  t.createBufferTracked({
    size: Uint32Array.BYTES_PER_ELEMENT,
    usage: GPUBufferUsage.COPY_SRC | GPUBufferUsage.STORAGE
  }),
  t.createBufferTracked({
    size: Uint32Array.BYTES_PER_ELEMENT,
    usage: GPUBufferUsage.COPY_SRC | GPUBufferUsage.STORAGE
  })];


  const descriptors = [
  {
    layout: 'auto',
    compute: {
      module,
      entryPoint: 'main1',
      constants: {
        c1: 1
      }
    }
  },
  {
    layout: 'auto',
    compute: {
      module,
      entryPoint: 'main2',
      constants: {
        c2: 2
      }
    }
  },
  {
    layout: 'auto',
    compute: {
      module,
      entryPoint: 'main3',
      constants: {
        // c3 is used as workgroup size
        c3: 1
      }
    }
  },
  {
    layout: 'auto',
    compute: {
      module,
      entryPoint: 'main1',
      constants: {
        // assign a different value to c1
        c1: 4
      }
    }
  }];


  const promises = t.params.isAsync ?
  Promise.all([
  t.device.createComputePipelineAsync(descriptors[0]),
  t.device.createComputePipelineAsync(descriptors[1]),
  t.device.createComputePipelineAsync(descriptors[2]),
  t.device.createComputePipelineAsync(descriptors[3])]
  ) :
  Promise.resolve([
  t.device.createComputePipeline(descriptors[0]),
  t.device.createComputePipeline(descriptors[1]),
  t.device.createComputePipeline(descriptors[2]),
  t.device.createComputePipeline(descriptors[3])]
  );

  const pipelines = await promises;
  const bindGroups = [
  t.device.createBindGroup({
    entries: [
    {
      binding: 0,
      resource: { buffer: buffers[0], offset: 0, size: Uint32Array.BYTES_PER_ELEMENT }
    }],

    layout: pipelines[0].getBindGroupLayout(0)
  }),
  t.device.createBindGroup({
    entries: [
    {
      binding: 0,
      resource: { buffer: buffers[1], offset: 0, size: Uint32Array.BYTES_PER_ELEMENT }
    }],

    layout: pipelines[1].getBindGroupLayout(0)
  }),
  t.device.createBindGroup({
    entries: [
    {
      binding: 0,
      resource: { buffer: buffers[2], offset: 0, size: Uint32Array.BYTES_PER_ELEMENT }
    }],

    layout: pipelines[2].getBindGroupLayout(0)
  }),
  t.device.createBindGroup({
    entries: [
    {
      binding: 0,
      resource: { buffer: buffers[3], offset: 0, size: Uint32Array.BYTES_PER_ELEMENT }
    }],

    layout: pipelines[3].getBindGroupLayout(0)
  })];


  const encoder = t.device.createCommandEncoder();
  const pass = encoder.beginComputePass();
  pass.setPipeline(pipelines[0]);
  pass.setBindGroup(0, bindGroups[0]);
  pass.dispatchWorkgroups(1);
  pass.setPipeline(pipelines[1]);
  pass.setBindGroup(0, bindGroups[1]);
  pass.dispatchWorkgroups(1);
  pass.setPipeline(pipelines[2]);
  pass.setBindGroup(0, bindGroups[2]);
  pass.dispatchWorkgroups(1);
  pass.setPipeline(pipelines[3]);
  pass.setBindGroup(0, bindGroups[3]);
  pass.dispatchWorkgroups(1);
  pass.end();
  t.device.queue.submit([encoder.finish()]);

  t.expectGPUBufferValuesEqual(buffers[0], expects[0]);
  t.expectGPUBufferValuesEqual(buffers[1], expects[1]);
  t.expectGPUBufferValuesEqual(buffers[2], expects[2]);
  t.expectGPUBufferValuesEqual(buffers[3], expects[3]);
});