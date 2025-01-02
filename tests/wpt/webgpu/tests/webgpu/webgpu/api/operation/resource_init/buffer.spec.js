/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/import { makeTestGroup } from '../../../../common/framework/test_group.js';import { unreachable } from '../../../../common/util/util.js';import { GPUConst } from '../../../constants.js';
import { GPUTest } from '../../../gpu_test.js';
import { getTextureCopyLayout } from '../../../util/texture/layout.js';


export const description = `
Test uninitialized buffers are initialized to zero when read
(or read-written, e.g. with depth write or atomics).

Note that:
-  We don't need 'copy_buffer_to_buffer_copy_destination' here because there has already been an
   operation test 'command_buffer.copyBufferToBuffer.single' that provides the same functionality.
`;

const kMapModeOptions = [GPUConst.MapMode.READ, GPUConst.MapMode.WRITE];
const kBufferUsagesForMappedAtCreationTests = [
GPUConst.BufferUsage.COPY_DST | GPUConst.BufferUsage.MAP_READ,
GPUConst.BufferUsage.COPY_SRC | GPUConst.BufferUsage.MAP_WRITE,
GPUConst.BufferUsage.COPY_SRC];


class F extends GPUTest {
  GetBufferUsageFromMapMode(mapMode) {
    switch (mapMode) {
      case GPUMapMode.READ:
        return GPUBufferUsage.COPY_DST | GPUBufferUsage.MAP_READ;
      case GPUMapMode.WRITE:
        return GPUBufferUsage.COPY_SRC | GPUBufferUsage.MAP_WRITE;
      default:
        unreachable();
        return 0;
    }
  }

  CheckGPUBufferContent(
  buffer,
  bufferUsage,
  expectedData)
  {
    const mappable = bufferUsage & GPUBufferUsage.MAP_READ;
    this.expectGPUBufferValuesEqual(buffer, expectedData, 0, { method: mappable ? 'map' : 'copy' });
  }

  TestBufferZeroInitInBindGroup(
  computeShaderModule,
  buffer,
  bufferOffset,
  boundBufferSize)
  {
    const computePipeline = this.device.createComputePipeline({
      layout: 'auto',
      compute: {
        module: computeShaderModule,
        entryPoint: 'main'
      }
    });
    const outputTexture = this.createTextureTracked({
      format: 'rgba8unorm',
      size: [1, 1, 1],
      usage: GPUTextureUsage.COPY_SRC | GPUTextureUsage.STORAGE_BINDING
    });
    const bindGroup = this.device.createBindGroup({
      layout: computePipeline.getBindGroupLayout(0),
      entries: [
      {
        binding: 0,
        resource: {
          buffer,
          offset: bufferOffset,
          size: boundBufferSize
        }
      },
      {
        binding: 1,
        resource: outputTexture.createView()
      }]

    });

    const encoder = this.device.createCommandEncoder();
    const computePass = encoder.beginComputePass();
    computePass.setBindGroup(0, bindGroup);
    computePass.setPipeline(computePipeline);
    computePass.dispatchWorkgroups(1);
    computePass.end();
    this.queue.submit([encoder.finish()]);

    this.CheckBufferAndOutputTexture(buffer, boundBufferSize + bufferOffset, outputTexture);
  }

  CreateRenderPipelineForTest(
  vertexShaderModule,
  testVertexBuffer)
  {
    const renderPipelineDescriptor = {
      layout: 'auto',
      vertex: {
        module: vertexShaderModule,
        entryPoint: 'main'
      },
      fragment: {
        module: this.device.createShaderModule({
          code: `
        @fragment
        fn main(@location(0) i_color : vec4<f32>) -> @location(0) vec4<f32> {
            return i_color;
        }`
        }),
        entryPoint: 'main',
        targets: [{ format: 'rgba8unorm' }]
      },
      primitive: {
        topology: 'point-list'
      }
    };
    if (testVertexBuffer) {
      renderPipelineDescriptor.vertex.buffers = [
      {
        arrayStride: 16,
        attributes: [{ format: 'float32x4', offset: 0, shaderLocation: 0 }]
      }];

    }

    return this.device.createRenderPipeline(renderPipelineDescriptor);
  }

  RecordInitializeTextureColor(
  encoder,
  texture,
  color)
  {
    const renderPass = encoder.beginRenderPass({
      colorAttachments: [
      {
        view: texture.createView(),
        clearValue: color,
        loadOp: 'clear',
        storeOp: 'store'
      }]

    });
    renderPass.end();
  }

  CheckBufferAndOutputTexture(
  buffer,
  bufferSize,
  outputTexture,
  outputTextureSize = [1, 1, 1],
  outputTextureColor = { R: 0.0, G: 1.0, B: 0.0, A: 1.0 })
  {
    this.expectSingleColor(outputTexture, 'rgba8unorm', {
      size: outputTextureSize,
      exp: outputTextureColor
    });

    const expectedBufferData = new Uint8Array(bufferSize);
    this.expectGPUBufferValuesEqual(buffer, expectedBufferData);
  }
}

export const g = makeTestGroup(F);

g.test('partial_write_buffer').
desc(
  `Verify when we upload data to a part of a buffer with writeBuffer() just after the creation of
the buffer, the remaining part of that buffer will be initialized to 0.`
).
paramsSubcasesOnly((u) => u.combine('offset', [0, 8, -12])).
fn((t) => {
  const { offset } = t.params;
  const bufferSize = 32;
  const appliedOffset = offset >= 0 ? offset : bufferSize + offset;

  const buffer = t.createBufferTracked({
    size: bufferSize,
    usage: GPUBufferUsage.COPY_SRC | GPUBufferUsage.COPY_DST
  });

  const copySize = 12;
  const writeData = new Uint8Array(copySize);
  const expectedData = new Uint8Array(bufferSize);
  for (let i = 0; i < copySize; ++i) {
    expectedData[appliedOffset + i] = writeData[i] = i + 1;
  }
  t.queue.writeBuffer(buffer, appliedOffset, writeData, 0);

  t.expectGPUBufferValuesEqual(buffer, expectedData);
});

g.test('map_whole_buffer').
desc(
  `Verify when we map the whole range of a mappable GPUBuffer to a typed array buffer just after
creating the GPUBuffer, the contents of both the typed array buffer and the GPUBuffer itself
have already been initialized to 0.`
).
params((u) => u.combine('mapMode', kMapModeOptions)).
fn(async (t) => {
  const { mapMode } = t.params;

  const bufferSize = 32;
  const bufferUsage = t.GetBufferUsageFromMapMode(mapMode);
  const buffer = t.createBufferTracked({
    size: bufferSize,
    usage: bufferUsage
  });

  await buffer.mapAsync(mapMode);
  const readData = new Uint8Array(buffer.getMappedRange());
  for (let i = 0; i < bufferSize; ++i) {
    t.expect(readData[i] === 0);
  }
  buffer.unmap();

  const expectedData = new Uint8Array(bufferSize);
  t.CheckGPUBufferContent(buffer, bufferUsage, expectedData);
});

g.test('map_partial_buffer').
desc(
  `Verify when we map a subrange of a mappable GPUBuffer to a typed array buffer just after the
creation of the GPUBuffer, the contents of both the typed array buffer and the GPUBuffer have
already been initialized to 0.`
).
params((u) => u.combine('mapMode', kMapModeOptions).beginSubcases().combine('offset', [0, 8, -16])).
fn(async (t) => {
  const { mapMode, offset } = t.params;
  const bufferSize = 32;
  const appliedOffset = offset >= 0 ? offset : bufferSize + offset;

  const bufferUsage = t.GetBufferUsageFromMapMode(mapMode);
  const buffer = t.createBufferTracked({
    size: bufferSize,
    usage: bufferUsage
  });

  const expectedData = new Uint8Array(bufferSize);
  {
    const mapSize = 16;
    await buffer.mapAsync(mapMode, appliedOffset, mapSize);
    const mappedData = new Uint8Array(buffer.getMappedRange(appliedOffset, mapSize));
    for (let i = 0; i < mapSize; ++i) {
      t.expect(mappedData[i] === 0);
      if (mapMode === GPUMapMode.WRITE) {
        mappedData[i] = expectedData[appliedOffset + i] = i + 1;
      }
    }
    buffer.unmap();
  }

  t.CheckGPUBufferContent(buffer, bufferUsage, expectedData);
});

g.test('mapped_at_creation_whole_buffer').
desc(
  `Verify when we call getMappedRange() at the whole range of a GPUBuffer created with
mappedAtCreation === true just after its creation, the contents of both the returned typed
array buffer of getMappedRange() and the GPUBuffer itself have all been initialized to 0.`
).
params((u) => u.combine('bufferUsage', kBufferUsagesForMappedAtCreationTests)).
fn((t) => {
  const { bufferUsage } = t.params;

  const bufferSize = 32;
  const buffer = t.createBufferTracked({
    mappedAtCreation: true,
    size: bufferSize,
    usage: bufferUsage
  });

  const mapped = new Uint8Array(buffer.getMappedRange());
  for (let i = 0; i < bufferSize; ++i) {
    t.expect(mapped[i] === 0);
  }
  buffer.unmap();

  const expectedData = new Uint8Array(bufferSize);
  t.CheckGPUBufferContent(buffer, bufferUsage, expectedData);
});

g.test('mapped_at_creation_partial_buffer').
desc(
  `Verify when we call getMappedRange() at a subrange of a GPUBuffer created with
mappedAtCreation === true just after its creation, the contents of both the returned typed
array buffer of getMappedRange() and the GPUBuffer itself have all been initialized to 0.`
).
params((u) =>
u.
combine('bufferUsage', kBufferUsagesForMappedAtCreationTests).
beginSubcases().
combine('offset', [0, 8, -16])
).
fn((t) => {
  const { bufferUsage, offset } = t.params;
  const bufferSize = 32;
  const appliedOffset = offset >= 0 ? offset : bufferSize + offset;

  const buffer = t.createBufferTracked({
    mappedAtCreation: true,
    size: bufferSize,
    usage: bufferUsage
  });

  const expectedData = new Uint8Array(bufferSize);
  {
    const mappedSize = 12;
    const mapped = new Uint8Array(buffer.getMappedRange(appliedOffset, mappedSize));
    for (let i = 0; i < mappedSize; ++i) {
      t.expect(mapped[i] === 0);
      if (!(bufferUsage & GPUBufferUsage.MAP_READ)) {
        mapped[i] = expectedData[appliedOffset + i] = i + 1;
      }
    }
    buffer.unmap();
  }

  t.CheckGPUBufferContent(buffer, bufferUsage, expectedData);
});

g.test('copy_buffer_to_buffer_copy_source').
desc(
  `Verify when the first usage of a GPUBuffer is being used as the source buffer of
CopyBufferToBuffer(), the contents of the GPUBuffer have already been initialized to 0.`
).
fn((t) => {
  const bufferSize = 32;
  const bufferUsage = GPUBufferUsage.COPY_SRC;
  const buffer = t.createBufferTracked({
    size: bufferSize,
    usage: bufferUsage
  });

  const expectedData = new Uint8Array(bufferSize);
  // copyBufferToBuffer() is called inside t.CheckGPUBufferContent().
  t.CheckGPUBufferContent(buffer, bufferUsage, expectedData);
});

g.test('copy_buffer_to_texture').
desc(
  `Verify when the first usage of a GPUBuffer is being used as the source buffer of
CopyBufferToTexture(), the contents of the GPUBuffer have already been initialized to 0.`
).
paramsSubcasesOnly((u) => u.combine('bufferOffset', [0, 8])).
fn((t) => {
  const { bufferOffset } = t.params;
  const textureSize = [8, 8, 1];
  const dstTextureFormat = 'rgba8unorm';

  const dstTexture = t.createTextureTracked({
    size: textureSize,
    format: dstTextureFormat,
    usage: GPUTextureUsage.COPY_SRC | GPUTextureUsage.COPY_DST
  });
  const layout = getTextureCopyLayout(dstTextureFormat, '2d', textureSize);
  const srcBufferSize = layout.byteLength + bufferOffset;
  const srcBufferUsage = GPUBufferUsage.COPY_SRC;
  const srcBuffer = t.createBufferTracked({
    size: srcBufferSize,
    usage: srcBufferUsage
  });

  const encoder = t.device.createCommandEncoder();
  encoder.copyBufferToTexture(
    {
      buffer: srcBuffer,
      offset: bufferOffset,
      bytesPerRow: layout.bytesPerRow,
      rowsPerImage: layout.rowsPerImage
    },
    { texture: dstTexture },
    textureSize
  );
  t.queue.submit([encoder.finish()]);

  t.CheckBufferAndOutputTexture(srcBuffer, srcBufferSize, dstTexture, textureSize, {
    R: 0.0,
    G: 0.0,
    B: 0.0,
    A: 0.0
  });
});

g.test('resolve_query_set_to_partial_buffer').
desc(
  `Verify when we resolve a query set into a GPUBuffer just after creating that GPUBuffer, the
remaining part of it will be initialized to 0.`
).
paramsSubcasesOnly((u) => u.combine('bufferOffset', [0, 256])).
fn((t) => {
  const { bufferOffset } = t.params;
  const bufferSize = bufferOffset + 8;
  const bufferUsage = GPUBufferUsage.COPY_SRC | GPUBufferUsage.QUERY_RESOLVE;
  const dstBuffer = t.createBufferTracked({
    size: bufferSize,
    usage: bufferUsage
  });

  const querySet = t.createQuerySetTracked({ type: 'occlusion', count: 1 });
  const encoder = t.device.createCommandEncoder();
  encoder.resolveQuerySet(querySet, 0, 1, dstBuffer, bufferOffset);
  t.queue.submit([encoder.finish()]);

  const expectedBufferData = new Uint8Array(bufferSize);
  t.CheckGPUBufferContent(dstBuffer, bufferUsage, expectedBufferData);
});

g.test('copy_texture_to_partial_buffer').
desc(
  `Verify when we copy from a GPUTexture into a GPUBuffer just after creating that GPUBuffer, the
remaining part of it will be initialized to 0.`
).
paramsSubcasesOnly((u) =>
u.
combine('bufferOffset', [0, 8, -16]).
combine('arrayLayerCount', [1, 3]).
combine('copyMipLevel', [0, 2]).
combine('rowsPerImage', [16, 20]).
filter((t) => {
  // We don't need to test the copies that will cover the whole GPUBuffer.
  return !(t.bufferOffset === 0 && t.rowsPerImage === 16);
})
).
fn((t) => {
  const { bufferOffset, arrayLayerCount, copyMipLevel, rowsPerImage } = t.params;
  const srcTextureFormat = 'r8uint';
  const textureSize = [32, 16, arrayLayerCount];

  const srcTexture = t.createTextureTracked({
    format: srcTextureFormat,
    usage: GPUTextureUsage.COPY_SRC | GPUTextureUsage.RENDER_ATTACHMENT,
    size: textureSize,
    mipLevelCount: copyMipLevel + 1
  });

  const bytesPerRow = 256;
  const layout = getTextureCopyLayout(srcTextureFormat, '2d', textureSize, {
    mipLevel: copyMipLevel,
    bytesPerRow,
    rowsPerImage
  });

  const dstBufferSize = layout.byteLength + Math.abs(bufferOffset);
  const dstBuffer = t.createBufferTracked({
    size: dstBufferSize,
    usage: GPUBufferUsage.COPY_SRC | GPUBufferUsage.COPY_DST
  });

  const encoder = t.device.createCommandEncoder();

  // Initialize srcTexture
  for (let layer = 0; layer < arrayLayerCount; ++layer) {
    const renderPass = encoder.beginRenderPass({
      colorAttachments: [
      {
        view: srcTexture.createView({
          baseArrayLayer: layer,
          arrayLayerCount: 1,
          baseMipLevel: copyMipLevel
        }),
        clearValue: { r: layer + 1, g: 0, b: 0, a: 0 },
        loadOp: 'clear',
        storeOp: 'store'
      }]

    });
    renderPass.end();
  }

  // Do texture-to-buffer copy
  const appliedOffset = Math.max(bufferOffset, 0);
  encoder.copyTextureToBuffer(
    { texture: srcTexture, mipLevel: copyMipLevel },
    { buffer: dstBuffer, offset: appliedOffset, bytesPerRow, rowsPerImage },
    layout.mipSize
  );
  t.queue.submit([encoder.finish()]);

  // Check if the contents of the destination buffer are what we expect.
  const expectedData = new Uint8Array(dstBufferSize);
  for (let layer = 0; layer < arrayLayerCount; ++layer) {
    for (let y = 0; y < layout.mipSize[1]; ++y) {
      for (let x = 0; x < layout.mipSize[0]; ++x) {
        expectedData[appliedOffset + layer * bytesPerRow * rowsPerImage + y * bytesPerRow + x] =
        layer + 1;
      }
    }
  }
  t.expectGPUBufferValuesEqual(dstBuffer, expectedData);
});

g.test('uniform_buffer').
desc(
  `Verify when we use a GPUBuffer as a uniform buffer just after the creation of that GPUBuffer,
    all the contents in that GPUBuffer have been initialized to 0.`
).
paramsSubcasesOnly((u) => u.combine('bufferOffset', [0, 256])).
fn((t) => {
  const { bufferOffset } = t.params;

  const boundBufferSize = 16;
  const buffer = t.createBufferTracked({
    size: bufferOffset + boundBufferSize,
    usage: GPUBufferUsage.COPY_SRC | GPUBufferUsage.UNIFORM
  });

  const computeShaderModule = t.device.createShaderModule({
    code: `
  struct UBO {
    value : vec4<u32>
  };
  @group(0) @binding(0) var<uniform> ubo : UBO;
  @group(0) @binding(1) var outImage : texture_storage_2d<rgba8unorm, write>;

  @compute @workgroup_size(1) fn main() {
      if (all(ubo.value == vec4<u32>(0u, 0u, 0u, 0u))) {
          textureStore(outImage, vec2<i32>(0, 0), vec4<f32>(0.0, 1.0, 0.0, 1.0));
      } else {
          textureStore(outImage, vec2<i32>(0, 0), vec4<f32>(1.0, 0.0, 0.0, 1.0));
      }
  }`
  });

  // Verify the whole range of the buffer has been initialized to 0 in a compute shader.
  t.TestBufferZeroInitInBindGroup(computeShaderModule, buffer, bufferOffset, boundBufferSize);
});

g.test('readonly_storage_buffer').
desc(
  `Verify when we use a GPUBuffer as a read-only storage buffer just after the creation of that
    GPUBuffer, all the contents in that GPUBuffer have been initialized to 0.`
).
paramsSubcasesOnly((u) => u.combine('bufferOffset', [0, 256])).
fn((t) => {
  const { bufferOffset } = t.params;
  const boundBufferSize = 16;
  const buffer = t.createBufferTracked({
    size: bufferOffset + boundBufferSize,
    usage: GPUBufferUsage.COPY_SRC | GPUBufferUsage.STORAGE
  });

  const computeShaderModule = t.device.createShaderModule({
    code: `
    struct SSBO {
      value : vec4<u32>
    };
    @group(0) @binding(0) var<storage, read> ssbo : SSBO;
    @group(0) @binding(1) var outImage : texture_storage_2d<rgba8unorm, write>;

    @compute @workgroup_size(1) fn main() {
        if (all(ssbo.value == vec4<u32>(0u, 0u, 0u, 0u))) {
            textureStore(outImage, vec2<i32>(0, 0), vec4<f32>(0.0, 1.0, 0.0, 1.0));
        } else {
            textureStore(outImage, vec2<i32>(0, 0), vec4<f32>(1.0, 0.0, 0.0, 1.0));
        }
    }`
  });

  // Verify the whole range of the buffer has been initialized to 0 in a compute shader.
  t.TestBufferZeroInitInBindGroup(computeShaderModule, buffer, bufferOffset, boundBufferSize);
});

g.test('storage_buffer').
desc(
  `Verify when we use a GPUBuffer as a storage buffer just after the creation of that
    GPUBuffer, all the contents in that GPUBuffer have been initialized to 0.`
).
paramsSubcasesOnly((u) => u.combine('bufferOffset', [0, 256])).
fn((t) => {
  const { bufferOffset } = t.params;
  const boundBufferSize = 16;
  const buffer = t.createBufferTracked({
    size: bufferOffset + boundBufferSize,
    usage: GPUBufferUsage.COPY_SRC | GPUBufferUsage.STORAGE
  });

  const computeShaderModule = t.device.createShaderModule({
    code: `
    struct SSBO {
      value : vec4<u32>
    };
    @group(0) @binding(0) var<storage, read_write> ssbo : SSBO;
    @group(0) @binding(1) var outImage : texture_storage_2d<rgba8unorm, write>;

    @compute @workgroup_size(1) fn main() {
        if (all(ssbo.value == vec4<u32>(0u, 0u, 0u, 0u))) {
            textureStore(outImage, vec2<i32>(0, 0), vec4<f32>(0.0, 1.0, 0.0, 1.0));
        } else {
            textureStore(outImage, vec2<i32>(0, 0), vec4<f32>(1.0, 0.0, 0.0, 1.0));
        }
    }`
  });

  // Verify the whole range of the buffer has been initialized to 0 in a compute shader.
  t.TestBufferZeroInitInBindGroup(computeShaderModule, buffer, bufferOffset, boundBufferSize);
});

g.test('vertex_buffer').
desc(
  `Verify when we use a GPUBuffer as a vertex buffer just after the creation of that
  GPUBuffer, all the contents in that GPUBuffer have been initialized to 0.`
).
paramsSubcasesOnly((u) => u.combine('bufferOffset', [0, 16])).
fn((t) => {
  const { bufferOffset } = t.params;

  const renderPipeline = t.CreateRenderPipelineForTest(
    t.device.createShaderModule({
      code: `
      struct VertexOut {
        @location(0) color : vec4<f32>,
        @builtin(position) position : vec4<f32>,
      };

      @vertex fn main(@location(0) pos : vec4<f32>) -> VertexOut {
        var output : VertexOut;
        if (all(pos == vec4<f32>(0.0, 0.0, 0.0, 0.0))) {
          output.color = vec4<f32>(0.0, 1.0, 0.0, 1.0);
        } else {
          output.color = vec4<f32>(1.0, 0.0, 0.0, 1.0);
        }
        output.position = vec4<f32>(0.0, 0.0, 0.0, 1.0);
        return output;
      }`
    }),
    true
  );

  const bufferSize = 16 + bufferOffset;
  const vertexBuffer = t.createBufferTracked({
    size: bufferSize,
    usage: GPUBufferUsage.VERTEX | GPUBufferUsage.COPY_SRC
  });

  const outputTexture = t.createTextureTracked({
    format: 'rgba8unorm',
    size: [1, 1, 1],
    usage: GPUTextureUsage.COPY_SRC | GPUTextureUsage.RENDER_ATTACHMENT
  });

  const encoder = t.device.createCommandEncoder();
  const renderPass = encoder.beginRenderPass({
    colorAttachments: [
    {
      view: outputTexture.createView(),
      clearValue: { r: 0.0, g: 0.0, b: 0.0, a: 0.0 },
      loadOp: 'clear',
      storeOp: 'store'
    }]

  });
  renderPass.setVertexBuffer(0, vertexBuffer, bufferOffset);
  renderPass.setPipeline(renderPipeline);
  renderPass.draw(1);
  renderPass.end();
  t.queue.submit([encoder.finish()]);

  t.CheckBufferAndOutputTexture(vertexBuffer, bufferSize, outputTexture);
});

g.test('index_buffer').
desc(
  `Verify when we use a GPUBuffer as an index buffer just after the creation of that
GPUBuffer, all the contents in that GPUBuffer have been initialized to 0.`
).
paramsSubcasesOnly((u) => u.combine('bufferOffset', [0, 16])).
fn((t) => {
  const { bufferOffset } = t.params;

  const renderPipeline = t.CreateRenderPipelineForTest(
    t.device.createShaderModule({
      code: `
    struct VertexOut {
      @location(0) color : vec4<f32>,
      @builtin(position) position : vec4<f32>,
    };

    @vertex
    fn main(@builtin(vertex_index) VertexIndex : u32) -> VertexOut {
      var output : VertexOut;
      if (VertexIndex == 0u) {
        output.color = vec4<f32>(0.0, 1.0, 0.0, 1.0);
      } else {
        output.color = vec4<f32>(1.0, 0.0, 0.0, 1.0);
      }
      output.position = vec4<f32>(0.0, 0.0, 0.0, 1.0);
      return output;
    }`
    }),
    false
  );

  // The size of GPUBuffer must be at least 4.
  const bufferSize = 4 + bufferOffset;
  const indexBuffer = t.createBufferTracked({
    size: bufferSize,
    usage: GPUBufferUsage.INDEX | GPUBufferUsage.COPY_SRC
  });

  const outputTexture = t.createTextureTracked({
    format: 'rgba8unorm',
    size: [1, 1, 1],
    usage: GPUTextureUsage.COPY_SRC | GPUTextureUsage.RENDER_ATTACHMENT
  });

  const encoder = t.device.createCommandEncoder();
  const renderPass = encoder.beginRenderPass({
    colorAttachments: [
    {
      view: outputTexture.createView(),
      clearValue: { r: 0.0, g: 0.0, b: 0.0, a: 0.0 },
      loadOp: 'clear',
      storeOp: 'store'
    }]

  });
  renderPass.setPipeline(renderPipeline);
  renderPass.setIndexBuffer(indexBuffer, 'uint16', bufferOffset, 4);
  renderPass.drawIndexed(1);
  renderPass.end();
  t.queue.submit([encoder.finish()]);

  t.CheckBufferAndOutputTexture(indexBuffer, bufferSize, outputTexture);
});

g.test('indirect_buffer_for_draw_indirect').
desc(
  `Verify when we use a GPUBuffer as an indirect buffer for drawIndirect() or
drawIndexedIndirect() just after the creation of that GPUBuffer, all the contents in that GPUBuffer
have been initialized to 0.`
).
params((u) =>
u.combine('test_indexed_draw', [true, false]).beginSubcases().combine('bufferOffset', [0, 16])
).
fn((t) => {
  const { test_indexed_draw, bufferOffset } = t.params;

  const renderPipeline = t.CreateRenderPipelineForTest(
    t.device.createShaderModule({
      code: `
    struct VertexOut {
      @location(0) color : vec4<f32>,
      @builtin(position) position : vec4<f32>,
    };

    @vertex fn main() -> VertexOut {
      var output : VertexOut;
      output.color = vec4<f32>(1.0, 0.0, 0.0, 1.0);
      output.position = vec4<f32>(0.0, 0.0, 0.0, 1.0);
      return output;
    }`
    }),
    false
  );

  const kDrawIndirectParametersSize = 16;
  const kDrawIndexedIndirectParametersSize = 20;
  const bufferSize =
  Math.max(kDrawIndirectParametersSize, kDrawIndexedIndirectParametersSize) + bufferOffset;
  const indirectBuffer = t.createBufferTracked({
    size: bufferSize,
    usage: GPUBufferUsage.COPY_SRC | GPUBufferUsage.INDIRECT
  });

  const outputTexture = t.createTextureTracked({
    format: 'rgba8unorm',
    size: [1, 1, 1],
    usage: GPUTextureUsage.COPY_SRC | GPUTextureUsage.RENDER_ATTACHMENT
  });

  // Initialize outputTexture to green.
  const encoder = t.device.createCommandEncoder();
  t.RecordInitializeTextureColor(encoder, outputTexture, { r: 0.0, g: 1.0, b: 0.0, a: 1.0 });

  const renderPass = encoder.beginRenderPass({
    colorAttachments: [
    {
      view: outputTexture.createView(),
      loadOp: 'load',
      storeOp: 'store'
    }]

  });
  renderPass.setPipeline(renderPipeline);

  let indexBuffer = undefined;
  if (test_indexed_draw) {
    indexBuffer = t.createBufferTracked({
      size: 4,
      usage: GPUBufferUsage.INDEX
    });
    renderPass.setIndexBuffer(indexBuffer, 'uint16');
    renderPass.drawIndexedIndirect(indirectBuffer, bufferOffset);
  } else {
    renderPass.drawIndirect(indirectBuffer, bufferOffset);
  }

  renderPass.end();
  t.queue.submit([encoder.finish()]);

  // The indirect buffer should be lazily cleared to 0, so we actually draw nothing and the color
  // attachment will keep its original color (green) after we end the render pass.
  t.CheckBufferAndOutputTexture(indirectBuffer, bufferSize, outputTexture);
});

g.test('indirect_buffer_for_dispatch_indirect').
desc(
  `Verify when we use a GPUBuffer as an indirect buffer for dispatchWorkgroupsIndirect() just
    after the creation of that GPUBuffer, all the contents in that GPUBuffer have been initialized
    to 0.`
).
paramsSubcasesOnly((u) => u.combine('bufferOffset', [0, 16])).
fn((t) => {
  const { bufferOffset } = t.params;

  const computePipeline = t.device.createComputePipeline({
    layout: 'auto',
    compute: {
      module: t.device.createShaderModule({
        code: `
        @group(0) @binding(0) var outImage : texture_storage_2d<rgba8unorm, write>;

        @compute @workgroup_size(1) fn main() {
          textureStore(outImage, vec2<i32>(0, 0), vec4<f32>(1.0, 0.0, 0.0, 1.0));
        }`
      }),
      entryPoint: 'main'
    }
  });

  const kDispatchIndirectParametersSize = 12;
  const bufferSize = kDispatchIndirectParametersSize + bufferOffset;
  const indirectBuffer = t.createBufferTracked({
    size: bufferSize,
    usage: GPUBufferUsage.COPY_SRC | GPUBufferUsage.INDIRECT
  });

  const outputTexture = t.createTextureTracked({
    format: 'rgba8unorm',
    size: [1, 1, 1],
    usage:
    GPUTextureUsage.COPY_SRC |
    GPUTextureUsage.RENDER_ATTACHMENT |
    GPUTextureUsage.STORAGE_BINDING
  });

  // Initialize outputTexture to green.
  const encoder = t.device.createCommandEncoder();
  t.RecordInitializeTextureColor(encoder, outputTexture, { r: 0.0, g: 1.0, b: 0.0, a: 1.0 });

  const bindGroup = t.device.createBindGroup({
    layout: computePipeline.getBindGroupLayout(0),
    entries: [
    {
      binding: 0,
      resource: outputTexture.createView()
    }]

  });

  // The indirect buffer should be lazily cleared to 0, so we actually don't execute the compute
  // shader and the output texture should keep its original color (green).
  const computePass = encoder.beginComputePass();
  computePass.setBindGroup(0, bindGroup);
  computePass.setPipeline(computePipeline);
  computePass.dispatchWorkgroupsIndirect(indirectBuffer, bufferOffset);
  computePass.end();
  t.queue.submit([encoder.finish()]);

  // The indirect buffer should be lazily cleared to 0, so we actually draw nothing and the color
  // attachment will keep its original color (green) after we end the compute pass.
  t.CheckBufferAndOutputTexture(indirectBuffer, bufferSize, outputTexture);
});