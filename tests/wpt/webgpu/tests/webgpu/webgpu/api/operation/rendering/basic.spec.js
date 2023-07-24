/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/ export const description = `
Basic command buffer rendering tests.
`;
import { makeTestGroup } from '../../../../common/framework/test_group.js';
import { now } from '../../../../common/util/util.js';
import { GPUTest } from '../../../gpu_test.js';
import { checkElementsEqual } from '../../../util/check_contents.js';

export const g = makeTestGroup(GPUTest);

g.test('clear').fn(t => {
  const dst = t.device.createBuffer({
    size: 4,
    usage: GPUBufferUsage.COPY_SRC | GPUBufferUsage.COPY_DST,
  });

  const colorAttachment = t.device.createTexture({
    format: 'rgba8unorm',
    size: { width: 1, height: 1, depthOrArrayLayers: 1 },
    usage: GPUTextureUsage.COPY_SRC | GPUTextureUsage.RENDER_ATTACHMENT,
  });
  const colorAttachmentView = colorAttachment.createView();

  const encoder = t.device.createCommandEncoder();
  const pass = encoder.beginRenderPass({
    colorAttachments: [
      {
        view: colorAttachmentView,
        clearValue: { r: 0.0, g: 1.0, b: 0.0, a: 1.0 },
        loadOp: 'clear',
        storeOp: 'store',
      },
    ],
  });
  pass.end();
  encoder.copyTextureToBuffer(
    { texture: colorAttachment, mipLevel: 0, origin: { x: 0, y: 0, z: 0 } },
    { buffer: dst, bytesPerRow: 256 },
    { width: 1, height: 1, depthOrArrayLayers: 1 }
  );

  t.device.queue.submit([encoder.finish()]);

  t.expectGPUBufferValuesEqual(dst, new Uint8Array([0x00, 0xff, 0x00, 0xff]));
});

g.test('fullscreen_quad').fn(t => {
  const dst = t.device.createBuffer({
    size: 4,
    usage: GPUBufferUsage.COPY_SRC | GPUBufferUsage.COPY_DST,
  });

  const colorAttachment = t.device.createTexture({
    format: 'rgba8unorm',
    size: { width: 1, height: 1, depthOrArrayLayers: 1 },
    usage: GPUTextureUsage.COPY_SRC | GPUTextureUsage.RENDER_ATTACHMENT,
  });
  const colorAttachmentView = colorAttachment.createView();

  const pipeline = t.device.createRenderPipeline({
    layout: 'auto',
    vertex: {
      module: t.device.createShaderModule({
        code: `
        @vertex fn main(
          @builtin(vertex_index) VertexIndex : u32
          ) -> @builtin(position) vec4<f32> {
            var pos : array<vec2<f32>, 3> = array<vec2<f32>, 3>(
                vec2<f32>(-1.0, -3.0),
                vec2<f32>(3.0, 1.0),
                vec2<f32>(-1.0, 1.0));
            return vec4<f32>(pos[VertexIndex], 0.0, 1.0);
          }
          `,
      }),
      entryPoint: 'main',
    },
    fragment: {
      module: t.device.createShaderModule({
        code: `
          @fragment fn main() -> @location(0) vec4<f32> {
            return vec4<f32>(0.0, 1.0, 0.0, 1.0);
          }
          `,
      }),
      entryPoint: 'main',
      targets: [{ format: 'rgba8unorm' }],
    },
    primitive: { topology: 'triangle-list' },
  });

  const encoder = t.device.createCommandEncoder();
  const pass = encoder.beginRenderPass({
    colorAttachments: [
      {
        view: colorAttachmentView,
        storeOp: 'store',
        clearValue: { r: 1.0, g: 0.0, b: 0.0, a: 1.0 },
        loadOp: 'clear',
      },
    ],
  });
  pass.setPipeline(pipeline);
  pass.draw(3);
  pass.end();
  encoder.copyTextureToBuffer(
    { texture: colorAttachment, mipLevel: 0, origin: { x: 0, y: 0, z: 0 } },
    { buffer: dst, bytesPerRow: 256 },
    { width: 1, height: 1, depthOrArrayLayers: 1 }
  );

  t.device.queue.submit([encoder.finish()]);

  t.expectGPUBufferValuesEqual(dst, new Uint8Array([0x00, 0xff, 0x00, 0xff]));
});

g.test('large_draw')
  .desc(
    `Test reasonably-sized large {draw, drawIndexed} (see also stress tests).

  Tests that draw calls behave reasonably with large vertex counts for
  non-indexed draws, large index counts for indexed draws, and large instance
  counts in both cases. Various combinations of these counts are tested with
  both direct and indirect draw calls.

  Draw call sizes are increased incrementally over these parameters until we the
  run out of values or completion of a draw call exceeds a fixed time limit of
  100ms.

  To validate that the drawn vertices actually made it though the pipeline on
  each draw call, we render a 3x3 target with the positions of the first and
  last vertices of the first and last instances in different respective corners,
  and everything else positioned to cover only one of the intermediate
  fragments. If the output image is completely yellow, then we can reasonably
  infer that all vertices were drawn.

  Params:
    - indexed= {true, false} - whether to test indexed or non-indexed draw calls
    - indirect= {true, false} - whether to use indirect or direct draw calls`
  )
  .params(u =>
    u //
      .combine('indexed', [true, false])
      .combine('indirect', [true, false])
  )
  .fn(async t => {
    const { indexed, indirect } = t.params;

    const kBytesPerRow = 256;
    const dst = t.device.createBuffer({
      size: 3 * kBytesPerRow,
      usage: GPUBufferUsage.COPY_SRC | GPUBufferUsage.COPY_DST,
    });

    const paramsBuffer = t.device.createBuffer({
      size: 8,
      usage: GPUBufferUsage.UNIFORM | GPUBufferUsage.COPY_DST,
    });

    const indirectBuffer = t.device.createBuffer({
      size: 20,
      usage: GPUBufferUsage.INDIRECT | GPUBufferUsage.COPY_DST,
    });
    const writeIndirectParams = (count, instanceCount) => {
      const params = new Uint32Array(5);
      params[0] = count; // Vertex or index count
      params[1] = instanceCount;
      params[2] = 0; // First vertex or index
      params[3] = 0; // First instance (non-indexed) or base vertex (indexed)
      params[4] = 0; // First instance (indexed)
      t.device.queue.writeBuffer(indirectBuffer, 0, params, 0, 5);
    };

    let indexBuffer = null;
    if (indexed) {
      const kMaxIndices = 16 * 1024 * 1024;
      indexBuffer = t.device.createBuffer({
        size: kMaxIndices * Uint32Array.BYTES_PER_ELEMENT,
        usage: GPUBufferUsage.INDEX | GPUBufferUsage.COPY_DST,
        mappedAtCreation: true,
      });
      t.trackForCleanup(indexBuffer);
      const indexData = new Uint32Array(indexBuffer.getMappedRange());
      for (let i = 0; i < kMaxIndices; ++i) {
        indexData[i] = i;
      }
      indexBuffer.unmap();
    }

    const colorAttachment = t.device.createTexture({
      format: 'rgba8unorm',
      size: { width: 3, height: 3, depthOrArrayLayers: 1 },
      usage: GPUTextureUsage.COPY_SRC | GPUTextureUsage.RENDER_ATTACHMENT,
    });
    const colorAttachmentView = colorAttachment.createView();

    const bgLayout = t.device.createBindGroupLayout({
      entries: [
        {
          binding: 0,
          visibility: GPUShaderStage.VERTEX,
          buffer: {},
        },
      ],
    });

    const bindGroup = t.device.createBindGroup({
      layout: bgLayout,
      entries: [
        {
          binding: 0,
          resource: { buffer: paramsBuffer },
        },
      ],
    });

    const pipeline = t.device.createRenderPipeline({
      layout: t.device.createPipelineLayout({ bindGroupLayouts: [bgLayout] }),

      vertex: {
        module: t.device.createShaderModule({
          code: `
          struct Params {
            numVertices: u32,
            numInstances: u32,
          };

          fn selectValue(index: u32, maxIndex: u32) -> f32 {
            let highOrMid = select(0.0, 2.0 / 3.0, index == maxIndex - 1u);
            return select(highOrMid, -2.0 / 3.0, index == 0u);
          }

          @group(0) @binding(0) var<uniform> params: Params;

          @vertex fn main(
              @builtin(vertex_index) v: u32,
              @builtin(instance_index) i: u32)
              -> @builtin(position) vec4<f32> {
            let x = selectValue(v, params.numVertices);
            let y = -selectValue(i, params.numInstances);
            return vec4<f32>(x, y, 0.0, 1.0);
          }
          `,
        }),
        entryPoint: 'main',
      },
      fragment: {
        module: t.device.createShaderModule({
          code: `
            @fragment fn main() -> @location(0) vec4<f32> {
              return vec4<f32>(1.0, 1.0, 0.0, 1.0);
            }
            `,
        }),
        entryPoint: 'main',
        targets: [{ format: 'rgba8unorm' }],
      },
      primitive: { topology: 'point-list' },
    });

    const runPipeline = (numVertices, numInstances) => {
      const encoder = t.device.createCommandEncoder();
      const pass = encoder.beginRenderPass({
        colorAttachments: [
          {
            view: colorAttachmentView,
            storeOp: 'store',
            clearValue: { r: 0.0, g: 0.0, b: 1.0, a: 1.0 },
            loadOp: 'clear',
          },
        ],
      });

      pass.setPipeline(pipeline);
      pass.setBindGroup(0, bindGroup);
      if (indexBuffer !== null) {
        pass.setIndexBuffer(indexBuffer, 'uint32');
      }

      if (indirect) {
        writeIndirectParams(numVertices, numInstances);
        if (indexed) {
          pass.drawIndexedIndirect(indirectBuffer, 0);
        } else {
          pass.drawIndirect(indirectBuffer, 0);
        }
      } else {
        if (indexed) {
          pass.drawIndexed(numVertices, numInstances);
        } else {
          pass.draw(numVertices, numInstances);
        }
      }
      pass.end();
      encoder.copyTextureToBuffer(
        { texture: colorAttachment, mipLevel: 0, origin: { x: 0, y: 0, z: 0 } },
        { buffer: dst, bytesPerRow: kBytesPerRow },
        { width: 3, height: 3, depthOrArrayLayers: 1 }
      );

      const params = new Uint32Array([numVertices, numInstances]);
      t.device.queue.writeBuffer(paramsBuffer, 0, params, 0, 2);
      t.device.queue.submit([encoder.finish()]);

      const yellow = [0xff, 0xff, 0x00, 0xff];
      const allYellow = new Uint8Array([...yellow, ...yellow, ...yellow]);
      for (const row of [0, 1, 2]) {
        t.expectGPUBufferValuesPassCheck(dst, data => checkElementsEqual(data, allYellow), {
          srcByteOffset: row * 256,
          type: Uint8Array,
          typedLength: 12,
        });
      }
    };

    // If any iteration takes longer than this, we stop incrementing along that
    // branch and move on to the next instance count. Note that the max
    // supported vertex count for any iteration is 2**24 due to our choice of
    // index buffer size.
    const maxDurationMs = 100;
    const counts = [
      {
        numInstances: 4,
        vertexCounts: [2 ** 10, 2 ** 16, 2 ** 18, 2 ** 20, 2 ** 22, 2 ** 24],
      },
      {
        numInstances: 2 ** 8,
        vertexCounts: [2 ** 10, 2 ** 16, 2 ** 18, 2 ** 20, 2 ** 22],
      },
      {
        numInstances: 2 ** 10,
        vertexCounts: [2 ** 8, 2 ** 10, 2 ** 12, 2 ** 16, 2 ** 18, 2 ** 20],
      },
      {
        numInstances: 2 ** 16,
        vertexCounts: [2 ** 4, 2 ** 8, 2 ** 10, 2 ** 12, 2 ** 14],
      },
      {
        numInstances: 2 ** 20,
        vertexCounts: [2 ** 4, 2 ** 8, 2 ** 10],
      },
    ];

    for (const { numInstances, vertexCounts } of counts) {
      for (const numVertices of vertexCounts) {
        const start = now();
        runPipeline(numVertices, numInstances);
        await t.device.queue.onSubmittedWorkDone();
        const duration = now() - start;
        if (duration >= maxDurationMs) {
          break;
        }
      }
    }
  });
