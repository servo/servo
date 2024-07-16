/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/import { assert, unreachable } from '../../../../../common/util/util.js';import { GPUTest } from '../../../../gpu_test.js';import { checkElementsEqualEither } from '../../../../util/check_contents.js';


export const kAllWriteOps = ['storage', 'b2b-copy', 't2b-copy', 'write-buffer'];

export const kAllReadOps = [
'input-vertex',
'input-index',
'input-indirect',
'input-indirect-index',
'input-indirect-dispatch',

'constant-uniform',

'storage-read',

'b2b-copy',
'b2t-copy'];











const kOpInfo =

{
  'write-buffer': {
    contexts: ['queue']
  },
  'b2t-copy': {
    contexts: ['command-encoder']
  },
  'b2b-copy': {
    contexts: ['command-encoder']
  },
  't2b-copy': {
    contexts: ['command-encoder']
  },
  storage: {
    contexts: ['compute-pass-encoder', 'render-pass-encoder', 'render-bundle-encoder']
  },
  'storage-read': {
    contexts: ['compute-pass-encoder', 'render-pass-encoder', 'render-bundle-encoder']
  },
  'input-vertex': {
    contexts: ['render-pass-encoder', 'render-bundle-encoder']
  },
  'input-index': {
    contexts: ['render-pass-encoder', 'render-bundle-encoder']
  },
  'input-indirect': {
    contexts: ['render-pass-encoder', 'render-bundle-encoder']
  },
  'input-indirect-index': {
    contexts: ['render-pass-encoder', 'render-bundle-encoder']
  },
  'input-indirect-dispatch': {
    contexts: ['compute-pass-encoder']
  },
  'constant-uniform': {
    contexts: ['render-pass-encoder', 'render-bundle-encoder']
  }
};

export function checkOpsValidForContext(
ops,
context)
{
  const valid =
  kOpInfo[ops[0]].contexts.includes(context[0]) && kOpInfo[ops[1]].contexts.includes(context[1]);
  if (!valid) return false;

  if (
  context[0] === 'render-bundle-encoder' ||
  context[0] === 'render-pass-encoder' ||
  context[1] === 'render-bundle-encoder' ||
  context[1] === 'render-pass-encoder')
  {
    // In a render pass, it is invalid to use a resource as both writable and another usage.
    // Also, for storage+storage usage, the application is opting into racy behavior.
    // The storage+storage case is also skipped as the results cannot be reliably tested.
    const checkImpl = (op1, op2) => {
      switch (op1) {
        case 'storage':
          switch (op2) {
            case 'storage':
            case 'storage-read':
            case 'input-vertex':
            case 'input-index':
            case 'input-indirect':
            case 'input-indirect-index':
            case 'constant-uniform':
              // Write+other, or racy.
              return false;
            case 'b2t-copy':
            case 't2b-copy':
            case 'b2b-copy':
            case 'write-buffer':
              // These don't occur in a render pass.
              return true;
          }
          break;
        case 'input-vertex':
        case 'input-index':
        case 'input-indirect':
        case 'input-indirect-index':
        case 'constant-uniform':
        case 'b2t-copy':
        case 't2b-copy':
        case 'b2b-copy':
        case 'write-buffer':
          // These are not write usages, or don't occur in a render pass.
          break;
      }
      return true;
    };
    return checkImpl(ops[0], ops[1]) && checkImpl(ops[1], ops[0]);
  }
  return true;
}

const kDummyVertexShader = `
@vertex fn vert_main() -> @builtin(position) vec4<f32> {
  return vec4<f32>(0.5, 0.5, 0.0, 1.0);
}
`;

// Note: If it would be useful to have any of these helpers be separate from the fixture,
// they can be refactored into standalone functions.
export class BufferSyncTest extends GPUTest {
  // Vertex and index buffers used in read render pass



  // Temp buffer and texture with values for buffer/texture copy write op
  // There can be at most 2 write op
  tmpValueBuffers = [undefined, undefined];
  tmpValueTextures = [undefined, undefined];

  // These intermediate buffers/textures are created before any read/write op
  // to avoid extra memory synchronization between ops introduced by await on buffer/texture creations.
  // Create extra buffers/textures needed by write operation
  async createIntermediateBuffersAndTexturesForWriteOp(
  writeOp,
  slot,
  value)
  {
    switch (writeOp) {
      case 'b2b-copy':
        this.tmpValueBuffers[slot] = await this.createBufferWithValue(value);
        break;
      case 't2b-copy':
        this.tmpValueTextures[slot] = await this.createTextureWithValue(value);
        break;
      default:
        break;
    }
  }

  // Create extra buffers/textures needed by read operation
  async createBuffersForReadOp(readOp, srcValue, opValue) {
    // This helps create values that will be written into dst buffer by the readop
    switch (readOp) {
      case 'input-index':
        // The index buffer will be the src buffer of the read op.
        // The src value for readOp will be 0
        // If the index buffer value is 0, the src value is written into the dst buffer.
        // If the index buffer value is 1, the op value is written into the dst buffer.
        this.vertexBuffer = await this.createBufferWithValues([srcValue, opValue]);
        break;
      case 'input-indirect':
        // The indirect buffer for the draw cmd will be the src buffer of the read op.
        // If the first value in the indirect buffer is 1, then the op value in vertex buffer will be written into dst buffer.
        // If the first value in indirect buffer is 0, then nothing will be write into dst buffer.
        this.vertexBuffer = await this.createBufferWithValues([opValue]);
        break;
      case 'input-indirect-index':
        // The indirect buffer for draw indexed cmd will be the src buffer of the read op.
        // If the first value in the indirect buffer is 1, then the opValue in vertex buffer will be written into dst buffer.
        // If the first value in indirect buffer is 0, then nothing will be write into dst buffer.
        this.vertexBuffer = await this.createBufferWithValues([opValue]);
        this.indexBuffer = await this.createBufferWithValues([0]);
        break;
      default:
        break;
    }

    let srcBuffer;
    switch (readOp) {
      case 'input-indirect':
        // vertexCount = {0, 1}
        // instanceCount = 1
        // firstVertex = 0
        // firstInstance = 0
        srcBuffer = await this.createBufferWithValues([srcValue, 1, 0, 0]);
        break;
      case 'input-indirect-index':
        // indexCount = {0, 1}
        // instanceCount = 1
        // firstIndex = 0
        // baseVertex = 0
        // firstInstance = 0
        srcBuffer = await this.createBufferWithValues([srcValue, 1, 0, 0, 0]);
        break;
      case 'input-indirect-dispatch':
        // workgroupCountX = {0, 1}
        // workgroupCountY = 1
        // workgroupCountZ = 1
        srcBuffer = await this.createBufferWithValues([srcValue, 1, 1]);
        break;
      default:
        srcBuffer = await this.createBufferWithValue(srcValue);
        break;
    }

    const dstBuffer = this.createBufferTracked({
      size: Uint32Array.BYTES_PER_ELEMENT,
      usage:
      GPUBufferUsage.COPY_SRC |
      GPUBufferUsage.COPY_DST |
      GPUBufferUsage.STORAGE |
      GPUBufferUsage.VERTEX |
      GPUBufferUsage.INDEX |
      GPUBufferUsage.INDIRECT |
      GPUBufferUsage.UNIFORM
    });

    return { srcBuffer, dstBuffer };
  }

  // Create a buffer with 1 uint32 element, and initialize it to a specified value.
  async createBufferWithValue(initValue) {
    const buffer = this.createBufferTracked({
      mappedAtCreation: true,
      size: Uint32Array.BYTES_PER_ELEMENT,
      usage:
      GPUBufferUsage.COPY_SRC |
      GPUBufferUsage.COPY_DST |
      GPUBufferUsage.STORAGE |
      GPUBufferUsage.VERTEX |
      GPUBufferUsage.INDEX |
      GPUBufferUsage.INDIRECT |
      GPUBufferUsage.UNIFORM
    });
    new Uint32Array(buffer.getMappedRange()).fill(initValue);
    buffer.unmap();
    await this.queue.onSubmittedWorkDone();
    return buffer;
  }

  // Create a buffer, and initialize it to the specified values.
  async createBufferWithValues(initValues) {
    const buffer = this.createBufferTracked({
      mappedAtCreation: true,
      size: Uint32Array.BYTES_PER_ELEMENT * initValues.length,
      usage:
      GPUBufferUsage.COPY_SRC |
      GPUBufferUsage.COPY_DST |
      GPUBufferUsage.STORAGE |
      GPUBufferUsage.VERTEX |
      GPUBufferUsage.INDEX |
      GPUBufferUsage.INDIRECT |
      GPUBufferUsage.UNIFORM
    });
    const bufferView = new Uint32Array(buffer.getMappedRange());
    bufferView.set(initValues);
    buffer.unmap();
    await this.queue.onSubmittedWorkDone();
    return buffer;
  }

  // Create a 1x1 texture, and initialize it to a specified value for all elements.
  async createTextureWithValue(initValue) {
    // This is not hot in profiles; optimize if this gets used more heavily.
    const data = new Uint32Array(1).fill(initValue);
    const texture = this.createTextureTracked({
      size: { width: 1, height: 1, depthOrArrayLayers: 1 },
      format: 'r32uint',
      usage: GPUTextureUsage.COPY_SRC | GPUTextureUsage.COPY_DST
    });
    this.device.queue.writeTexture(
      { texture, mipLevel: 0, origin: { x: 0, y: 0, z: 0 } },
      data,
      { offset: 0, bytesPerRow: 256, rowsPerImage: 1 },
      { width: 1, height: 1, depthOrArrayLayers: 1 }
    );
    await this.queue.onSubmittedWorkDone();
    return texture;
  }

  createBindGroup(
  pipeline,
  buffer)
  {
    return this.device.createBindGroup({
      layout: pipeline.getBindGroupLayout(0),
      entries: [{ binding: 0, resource: { buffer } }]
    });
  }

  // Create a compute pipeline and write given data into storage buffer.
  createStorageWriteComputePipeline(value) {
    const wgslCompute = `
      struct Data {
        a : u32
      };

      @group(0) @binding(0) var<storage, read_write> data : Data;
      @compute @workgroup_size(1) fn main() {
        data.a = ${value}u;
      }
    `;

    return this.device.createComputePipeline({
      layout: 'auto',
      compute: {
        module: this.device.createShaderModule({
          code: wgslCompute
        }),
        entryPoint: 'main'
      }
    });
  }

  createTrivialRenderPipeline(wgslShaders) {
    return this.device.createRenderPipeline({
      layout: 'auto',
      vertex: {
        module: this.device.createShaderModule({
          code: wgslShaders.vertex
        }),
        entryPoint: 'vert_main'
      },
      fragment: {
        module: this.device.createShaderModule({
          code: wgslShaders.fragment
        }),
        entryPoint: 'frag_main',
        targets: [{ format: 'rgba8unorm' }]
      },
      primitive: { topology: 'point-list' }
    });
  }

  // Create a render pipeline and write given data into storage buffer at fragment stage.
  createStorageWriteRenderPipeline(value) {
    const wgslShaders = {
      vertex: kDummyVertexShader,
      fragment: `
      struct Data {
        a : u32
      };

      @group(0) @binding(0) var<storage, read_write> data : Data;
      @fragment fn frag_main() -> @location(0) vec4<f32> {
        data.a = ${value}u;
        return vec4<f32>();  // result does't matter
      }
    `
    };

    return this.createTrivialRenderPipeline(wgslShaders);
  }

  beginSimpleRenderPass(encoder) {
    const view = this.createTextureTracked({
      size: { width: 1, height: 1, depthOrArrayLayers: 1 },
      format: 'rgba8unorm',
      usage: GPUTextureUsage.RENDER_ATTACHMENT
    }).createView();
    return encoder.beginRenderPass({
      colorAttachments: [
      {
        view,
        clearValue: { r: 0.0, g: 1.0, b: 0.0, a: 1.0 },
        loadOp: 'clear',
        storeOp: 'store'
      }]

    });
  }

  // Write buffer via draw call in render pass. Use bundle if needed.
  encodeWriteAsStorageBufferInRenderPass(
  renderer,
  buffer,
  value)
  {
    const pipeline = this.createStorageWriteRenderPipeline(value);
    const bindGroup = this.createBindGroup(pipeline, buffer);

    renderer.setBindGroup(0, bindGroup);
    renderer.setPipeline(pipeline);
    renderer.draw(1, 1, 0, 0);
  }

  // Write buffer via dispatch call in compute pass.
  encodeWriteAsStorageBufferInComputePass(
  pass,
  buffer,
  value)
  {
    const pipeline = this.createStorageWriteComputePipeline(value);
    const bindGroup = this.createBindGroup(pipeline, buffer);
    pass.setPipeline(pipeline);
    pass.setBindGroup(0, bindGroup);
    pass.dispatchWorkgroups(1);
  }

  // Write buffer via BufferToBuffer copy.
  encodeWriteByB2BCopy(encoder, buffer, slot) {
    const tmpBuffer = this.tmpValueBuffers[slot];
    assert(tmpBuffer !== undefined);
    // The write operation via b2b copy is just encoded into command encoder, it doesn't write immediately.
    encoder.copyBufferToBuffer(tmpBuffer, 0, buffer, 0, Uint32Array.BYTES_PER_ELEMENT);
  }

  // Write buffer via TextureToBuffer copy.
  encodeWriteByT2BCopy(encoder, buffer, slot) {
    const tmpTexture = this.tmpValueTextures[slot];
    assert(tmpTexture !== undefined);
    // The write operation via t2b copy is just encoded into command encoder, it doesn't write immediately.
    encoder.copyTextureToBuffer(
      { texture: tmpTexture, mipLevel: 0, origin: { x: 0, y: 0, z: 0 } },
      { buffer, bytesPerRow: 256 },
      { width: 1, height: 1, depthOrArrayLayers: 1 }
    );
  }

  // Write buffer via writeBuffer API on queue
  writeByWriteBuffer(buffer, value) {
    // This is not hot in profiles; optimize if this gets used more heavily.
    const data = new Uint32Array(1).fill(value);
    this.device.queue.writeBuffer(buffer, 0, data);
  }

  // Issue write operation via render pass, compute pass, copy, etc.
  encodeWriteOp(
  helper,
  operation,
  context,
  buffer,
  writeOpSlot,
  value)
  {
    helper.ensureContext(context);

    switch (operation) {
      case 'write-buffer':
        this.writeByWriteBuffer(buffer, value);
        break;
      case 'storage':
        switch (context) {
          case 'render-pass-encoder':
            assert(helper.renderPassEncoder !== undefined);
            this.encodeWriteAsStorageBufferInRenderPass(helper.renderPassEncoder, buffer, value);
            break;
          case 'render-bundle-encoder':
            assert(helper.renderBundleEncoder !== undefined);
            this.encodeWriteAsStorageBufferInRenderPass(helper.renderBundleEncoder, buffer, value);
            break;
          case 'compute-pass-encoder':
            assert(helper.computePassEncoder !== undefined);
            this.encodeWriteAsStorageBufferInComputePass(helper.computePassEncoder, buffer, value);
            break;
          default:
            unreachable();
        }
        break;
      case 'b2b-copy':
        assert(helper.commandEncoder !== undefined);
        this.encodeWriteByB2BCopy(helper.commandEncoder, buffer, writeOpSlot);
        break;
      case 't2b-copy':
        assert(helper.commandEncoder !== undefined);
        this.encodeWriteByT2BCopy(helper.commandEncoder, buffer, writeOpSlot);
        break;
      default:
        unreachable();
    }
  }

  // Create a compute pipeline: read from src buffer and write it into the storage buffer.
  createStorageReadComputePipeline() {
    const wgslCompute = `
      struct Data {
        a : u32
      };

      @group(0) @binding(0) var<storage, read> srcData : Data;
      @group(0) @binding(1) var<storage, read_write> dstData : Data;

      @compute @workgroup_size(1) fn main() {
        dstData.a = srcData.a;
      }
    `;

    return this.device.createComputePipeline({
      layout: 'auto',
      compute: {
        module: this.device.createShaderModule({
          code: wgslCompute
        }),
        entryPoint: 'main'
      }
    });
  }

  createBindGroupSrcDstBuffer(
  pipeline,
  srcBuffer,
  dstBuffer)
  {
    return this.device.createBindGroup({
      layout: pipeline.getBindGroupLayout(0),
      entries: [
      { binding: 0, resource: { buffer: srcBuffer } },
      { binding: 1, resource: { buffer: dstBuffer } }]

    });
  }

  // Create a render pipeline: read from vertex/index buffer and write it into the storage dst buffer at fragment stage.
  createVertexReadRenderPipeline() {
    const wgslShaders = {
      vertex: `
      struct VertexOutput {
        @builtin(position) position : vec4<f32>,
        @location(0) @interpolate(flat, either) data : u32,
      };

      @vertex fn vert_main(@location(0) input: u32) -> VertexOutput {
        var output : VertexOutput;
        output.position = vec4<f32>(0.5, 0.5, 0.0, 1.0);
        output.data = input;
        return output;
      }
      `,
      fragment: `
      struct Data {
        a : u32
      };

      @group(0) @binding(0) var<storage, read_write> data : Data;

      @fragment fn frag_main(@location(0) @interpolate(flat, either) input : u32) -> @location(0) vec4<f32> {
        data.a = input;
        return vec4<f32>();  // result does't matter
      }
      `
    };

    return this.device.createRenderPipeline({
      layout: 'auto',
      vertex: {
        module: this.device.createShaderModule({
          code: wgslShaders.vertex
        }),
        entryPoint: 'vert_main',
        buffers: [
        {
          arrayStride: Uint32Array.BYTES_PER_ELEMENT,
          attributes: [
          {
            shaderLocation: 0,
            offset: 0,
            format: 'uint32'
          }]

        }]

      },
      fragment: {
        module: this.device.createShaderModule({
          code: wgslShaders.fragment
        }),
        entryPoint: 'frag_main',
        targets: [{ format: 'rgba8unorm' }]
      },
      primitive: { topology: 'point-list' }
    });
  }

  // Create a render pipeline: read from uniform buffer and write it into the storage dst buffer at fragment stage.
  createUniformReadRenderPipeline() {
    const wgslShaders = {
      vertex: kDummyVertexShader,
      fragment: `
      struct Data {
        a : u32
      };

      @group(0) @binding(0) var<uniform> constant: Data;
      @group(0) @binding(1) var<storage, read_write> data : Data;

      @fragment fn frag_main() -> @location(0) vec4<f32> {
        data.a = constant.a;
        return vec4<f32>();  // result does't matter
      }
      `
    };

    return this.createTrivialRenderPipeline(wgslShaders);
  }

  // Create a render pipeline: read from storage src buffer and write it into the storage dst buffer at fragment stage.
  createStorageReadRenderPipeline() {
    const wgslShaders = {
      vertex: kDummyVertexShader,
      fragment: `
        struct Data {
          a : u32
        };

        @group(0) @binding(0) var<storage, read> srcData : Data;
        @group(0) @binding(1) var<storage, read_write> dstData : Data;

        @fragment fn frag_main() -> @location(0) vec4<f32> {
          dstData.a = srcData.a;
          return vec4<f32>();  // result does't matter
        }
      `
    };

    return this.device.createRenderPipeline({
      layout: 'auto',
      vertex: {
        module: this.device.createShaderModule({
          code: wgslShaders.vertex
        }),
        entryPoint: 'vert_main'
      },
      fragment: {
        module: this.device.createShaderModule({
          code: wgslShaders.fragment
        }),
        entryPoint: 'frag_main',
        targets: [{ format: 'rgba8unorm' }]
      },
      primitive: { topology: 'point-list' }
    });
  }

  // Write buffer via dispatch call in compute pass.
  encodeReadAsStorageBufferInComputePass(
  pass,
  srcBuffer,
  dstBuffer)
  {
    const pipeline = this.createStorageReadComputePipeline();
    const bindGroup = this.createBindGroupSrcDstBuffer(pipeline, srcBuffer, dstBuffer);
    pass.setPipeline(pipeline);
    pass.setBindGroup(0, bindGroup);
    pass.dispatchWorkgroups(1);
  }

  // Write buffer via dispatchWorkgroupsIndirect call in compute pass.
  encodeReadAsIndirectBufferInComputePass(
  pass,
  srcBuffer,
  dstBuffer,
  value)
  {
    const pipeline = this.createStorageWriteComputePipeline(value);
    const bindGroup = this.createBindGroup(pipeline, dstBuffer);
    pass.setPipeline(pipeline);
    pass.setBindGroup(0, bindGroup);
    pass.dispatchWorkgroupsIndirect(srcBuffer, 0);
  }

  // Read as vertex input and write buffer via draw call in render pass. Use bundle if needed.
  encodeReadAsVertexBufferInRenderPass(
  renderer,
  srcBuffer,
  dstBuffer)
  {
    const pipeline = this.createVertexReadRenderPipeline();
    const bindGroup = this.device.createBindGroup({
      layout: pipeline.getBindGroupLayout(0),
      entries: [{ binding: 0, resource: { buffer: dstBuffer } }]
    });

    renderer.setBindGroup(0, bindGroup);
    renderer.setPipeline(pipeline);
    renderer.setVertexBuffer(0, srcBuffer);
    renderer.draw(1);
  }

  // Read as index input and write buffer via draw call in render pass. Use bundle if needed.
  encodeReadAsIndexBufferInRenderPass(
  renderer,
  srcBuffer,
  dstBuffer,
  vertexBuffer)
  {
    const pipeline = this.createVertexReadRenderPipeline();
    const bindGroup = this.device.createBindGroup({
      layout: pipeline.getBindGroupLayout(0),
      entries: [{ binding: 0, resource: { buffer: dstBuffer } }]
    });

    renderer.setBindGroup(0, bindGroup);
    renderer.setPipeline(pipeline);
    renderer.setVertexBuffer(0, vertexBuffer);
    renderer.setIndexBuffer(srcBuffer, 'uint32');
    renderer.drawIndexed(1);
  }

  // Read as indirect input and write buffer via draw call in render pass. Use bundle if needed.
  encodeReadAsIndirectBufferInRenderPass(
  renderer,
  srcBuffer,
  dstBuffer,
  vertexBuffer)
  {
    const pipeline = this.createVertexReadRenderPipeline();
    const bindGroup = this.device.createBindGroup({
      layout: pipeline.getBindGroupLayout(0),
      entries: [{ binding: 0, resource: { buffer: dstBuffer } }]
    });

    renderer.setBindGroup(0, bindGroup);
    renderer.setPipeline(pipeline);
    renderer.setVertexBuffer(0, vertexBuffer);
    renderer.drawIndirect(srcBuffer, 0);
  }

  // Read as indexed indirect input and write buffer via draw call in render pass. Use bundle if needed.
  encodeReadAsIndexedIndirectBufferInRenderPass(
  renderer,
  srcBuffer,
  dstBuffer,
  vertexBuffer,
  indexBuffer)
  {
    const pipeline = this.createVertexReadRenderPipeline();
    const bindGroup = this.device.createBindGroup({
      layout: pipeline.getBindGroupLayout(0),
      entries: [{ binding: 0, resource: { buffer: dstBuffer } }]
    });

    renderer.setBindGroup(0, bindGroup);
    renderer.setPipeline(pipeline);
    renderer.setVertexBuffer(0, vertexBuffer);
    renderer.setIndexBuffer(indexBuffer, 'uint32');
    renderer.drawIndexedIndirect(srcBuffer, 0);
  }

  // Read as uniform buffer and write buffer via draw call in render pass. Use bundle if needed.
  encodeReadAsUniformBufferInRenderPass(
  renderer,
  srcBuffer,
  dstBuffer)
  {
    const pipeline = this.createUniformReadRenderPipeline();
    const bindGroup = this.device.createBindGroup({
      layout: pipeline.getBindGroupLayout(0),
      entries: [
      { binding: 0, resource: { buffer: srcBuffer } },
      { binding: 1, resource: { buffer: dstBuffer } }]

    });

    renderer.setBindGroup(0, bindGroup);
    renderer.setPipeline(pipeline);
    renderer.draw(1);
  }

  // Read as storage buffer and write buffer via draw call in render pass. Use bundle if needed.
  encodeReadAsStorageBufferInRenderPass(
  renderer,
  srcBuffer,
  dstBuffer)
  {
    const pipeline = this.createStorageReadRenderPipeline();
    const bindGroup = this.createBindGroupSrcDstBuffer(pipeline, srcBuffer, dstBuffer);

    renderer.setBindGroup(0, bindGroup);
    renderer.setPipeline(pipeline);
    renderer.draw(1, 1, 0, 0);
  }

  // Read and write via BufferToBuffer copy.
  encodeReadByB2BCopy(encoder, srcBuffer, dstBuffer) {
    // The b2b copy is just encoded into command encoder, it doesn't write immediately.
    encoder.copyBufferToBuffer(srcBuffer, 0, dstBuffer, 0, Uint32Array.BYTES_PER_ELEMENT);
  }

  // Read and Write texture via BufferToTexture copy.
  encodeReadByB2TCopy(encoder, srcBuffer, dstBuffer) {
    const tmpTexture = this.createTextureTracked({
      size: { width: 1, height: 1, depthOrArrayLayers: 1 },
      format: 'r32uint',
      usage: GPUTextureUsage.COPY_SRC | GPUTextureUsage.COPY_DST
    });

    // The b2t copy is just encoded into command encoder, it doesn't write immediately.
    encoder.copyBufferToTexture(
      { buffer: srcBuffer, bytesPerRow: 256 },
      { texture: tmpTexture, mipLevel: 0, origin: { x: 0, y: 0, z: 0 } },
      { width: 1, height: 1, depthOrArrayLayers: 1 }
    );
    // The t2b copy is just encoded into command encoder, it doesn't write immediately.
    encoder.copyTextureToBuffer(
      { texture: tmpTexture, mipLevel: 0, origin: { x: 0, y: 0, z: 0 } },
      { buffer: dstBuffer, bytesPerRow: 256 },
      { width: 1, height: 1, depthOrArrayLayers: 1 }
    );
  }

  encodeReadOp(
  helper,
  operation,
  context,
  srcBuffer,
  dstBuffer)
  {
    helper.ensureContext(context);

    const renderer =
    context === 'render-bundle-encoder' ? helper.renderBundleEncoder : helper.renderPassEncoder;
    const computePass = context === 'compute-pass-encoder' ? helper.computePassEncoder : undefined;

    switch (operation) {
      case 'input-vertex':
        // The srcBuffer is used as vertexBuffer.
        // draw writes the same value in srcBuffer[0] to dstBuffer[0].
        assert(renderer !== undefined);
        this.encodeReadAsVertexBufferInRenderPass(renderer, srcBuffer, dstBuffer);
        break;
      case 'input-index':
        // The srcBuffer is used as indexBuffer.
        // With this vertexBuffer, drawIndexed writes the same value in srcBuffer[0] to dstBuffer[0].
        assert(renderer !== undefined);
        assert(this.vertexBuffer !== undefined);
        this.encodeReadAsIndexBufferInRenderPass(renderer, srcBuffer, dstBuffer, this.vertexBuffer);
        break;
      case 'input-indirect':
        // The srcBuffer is used as indirectBuffer for drawIndirect.
        // srcBuffer[0] = 0 or 1 (vertexCount), which will decide the value written into dstBuffer to be either 0 or 1.
        assert(renderer !== undefined);
        assert(this.vertexBuffer !== undefined);
        this.encodeReadAsIndirectBufferInRenderPass(
          renderer,
          srcBuffer,
          dstBuffer,
          this.vertexBuffer
        );
        break;
      case 'input-indirect-index':
        // The srcBuffer is used as indirectBuffer for drawIndexedIndirect.
        // srcBuffer[0] = 0 or 1 (indexCount), which will decide the value written into dstBuffer to be either 0 or 1.
        assert(renderer !== undefined);
        assert(this.vertexBuffer !== undefined);
        assert(this.indexBuffer !== undefined);
        this.encodeReadAsIndexedIndirectBufferInRenderPass(
          renderer,
          srcBuffer,
          dstBuffer,
          this.vertexBuffer,
          this.indexBuffer
        );
        break;
      case 'input-indirect-dispatch':
        // The srcBuffer is used as indirectBuffer for dispatch.
        // srcBuffer[0] = 0 or 1 (workgroupCountX), which will decide the value written into dstBuffer to be either 0 or 1.
        assert(computePass !== undefined);
        this.encodeReadAsIndirectBufferInComputePass(computePass, srcBuffer, dstBuffer, 1);
        break;
      case 'constant-uniform':
        // The srcBuffer is used as uniform buffer.
        assert(renderer !== undefined);
        this.encodeReadAsUniformBufferInRenderPass(renderer, srcBuffer, dstBuffer);
        break;
      case 'storage-read':
        switch (context) {
          case 'render-pass-encoder':
          case 'render-bundle-encoder':
            assert(renderer !== undefined);
            this.encodeReadAsStorageBufferInRenderPass(renderer, srcBuffer, dstBuffer);
            break;
          case 'compute-pass-encoder':
            assert(computePass !== undefined);
            this.encodeReadAsStorageBufferInComputePass(computePass, srcBuffer, dstBuffer);
            break;
          default:
            unreachable();
        }
        break;
      case 'b2b-copy':
        assert(helper.commandEncoder !== undefined);
        this.encodeReadByB2BCopy(helper.commandEncoder, srcBuffer, dstBuffer);
        break;
      case 'b2t-copy':
        assert(helper.commandEncoder !== undefined);
        this.encodeReadByB2TCopy(helper.commandEncoder, srcBuffer, dstBuffer);
        break;
      default:
        unreachable();
    }
  }

  verifyData(buffer, expectedValue) {
    // This is not hot in profiles; optimize if this gets used more heavily.
    const bufferData = new Uint32Array(1);
    bufferData[0] = expectedValue;
    this.expectGPUBufferValuesEqual(buffer, bufferData);
  }

  verifyDataTwoValidValues(buffer, expectedValue1, expectedValue2) {
    // This is not hot in profiles; optimize if this gets used more heavily.
    const bufferData1 = new Uint32Array(1);
    bufferData1[0] = expectedValue1;
    const bufferData2 = new Uint32Array(1);
    bufferData2[0] = expectedValue2;
    this.expectGPUBufferValuesPassCheck(
      buffer,
      (a) => checkElementsEqualEither(a, [bufferData1, bufferData2]),
      { type: Uint32Array, typedLength: 1 }
    );
  }
}