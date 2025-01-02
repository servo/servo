/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Tests for the behavior of read-write storage textures.

TODO:
- Test resource usage transitions with read-write storage textures
`;import { makeTestGroup } from '../../../../common/framework/test_group.js';
import { assert, unreachable } from '../../../../common/util/util.js';
import { kTextureDimensions } from '../../../capability_info.js';
import { kColorTextureFormats, kTextureFormatInfo } from '../../../format_info.js';
import { GPUTest } from '../../../gpu_test.js';
import { align } from '../../../util/math.js';

const kShaderStagesForReadWriteStorageTexture = ['fragment', 'compute'];



class F extends GPUTest {
  GetInitialData(storageTexture) {
    const format = storageTexture.format;
    const bytesPerBlock = kTextureFormatInfo[format].bytesPerBlock;
    assert(bytesPerBlock !== undefined);

    const width = storageTexture.width;
    const height = storageTexture.height;
    const depthOrArrayLayers = storageTexture.depthOrArrayLayers;
    const initialData = new ArrayBuffer(bytesPerBlock * width * height * depthOrArrayLayers);
    const initialTypedData = this.GetTypedArrayBuffer(initialData, format);
    for (let z = 0; z < depthOrArrayLayers; ++z) {
      for (let y = 0; y < height; ++y) {
        for (let x = 0; x < width; ++x) {
          const index = z * width * height + y * width + x;
          switch (format) {
            case 'r32sint':
              initialTypedData[index] = (index & 1 ? 1 : -1) * (2 * index + 1);
              break;
            case 'r32uint':
              initialTypedData[index] = 2 * index + 1;
              break;
            case 'r32float':
              initialTypedData[index] = (2 * index + 1) / 10.0;
              break;
          }
        }
      }
    }
    return initialData;
  }

  GetTypedArrayBuffer(arrayBuffer, format) {
    switch (format) {
      case 'r32sint':
        return new Int32Array(arrayBuffer);
      case 'r32uint':
        return new Uint32Array(arrayBuffer);
      case 'r32float':
        return new Float32Array(arrayBuffer);
      default:
        unreachable();
        return new Uint8Array(arrayBuffer);
    }
  }

  GetExpectedData(
  shaderStage,
  storageTexture,
  initialData)
  {
    const format = storageTexture.format;
    const bytesPerBlock = kTextureFormatInfo[format].bytesPerBlock;
    assert(bytesPerBlock !== undefined);

    const width = storageTexture.width;
    const height = storageTexture.height;
    const depthOrArrayLayers = storageTexture.depthOrArrayLayers;
    const bytesPerRowAlignment = align(bytesPerBlock * width, 256);
    const itemsPerRow = bytesPerRowAlignment / bytesPerBlock;

    const expectedData = new ArrayBuffer(
      bytesPerRowAlignment * (height * depthOrArrayLayers - 1) + bytesPerBlock * width
    );
    const expectedTypedData = this.GetTypedArrayBuffer(expectedData, format);
    const initialTypedData = this.GetTypedArrayBuffer(initialData, format);
    for (let z = 0; z < depthOrArrayLayers; ++z) {
      for (let y = 0; y < height; ++y) {
        for (let x = 0; x < width; ++x) {
          const expectedIndex = z * itemsPerRow * height + y * itemsPerRow + x;
          switch (shaderStage) {
            case 'compute':{
                // In the compute shader we flip the texture along the diagonal.
                const initialIndex =
                (depthOrArrayLayers - 1 - z) * width * height +
                (height - 1 - y) * width + (
                width - 1 - x);
                expectedTypedData[expectedIndex] = initialTypedData[initialIndex];
                break;
              }
            case 'fragment':{
                // In the fragment shader we double the original texel value of the read-write storage
                // texture.
                const initialIndex = z * width * height + y * width + x;
                expectedTypedData[expectedIndex] = initialTypedData[initialIndex] * 2;
                break;
              }
          }
        }
      }
    }
    return expectedData;
  }

  RecordCommandsToTransform(
  device,
  shaderStage,
  commandEncoder,
  rwTexture)
  {
    let declaration = '';
    switch (rwTexture.dimension) {
      case '1d':
        declaration = 'texture_storage_1d';
        break;
      case '2d':
        declaration =
        rwTexture.depthOrArrayLayers > 1 ? 'texture_storage_2d_array' : 'texture_storage_2d';
        break;
      case '3d':
        declaration = 'texture_storage_3d';
        break;
    }
    const textureDeclaration = `
    @group(0) @binding(0) var rwTexture: ${declaration}<${rwTexture.format}, read_write>;
    `;

    switch (shaderStage) {
      case 'fragment':{
          const vertexShader = `
        @vertex
        fn main(@builtin(vertex_index) VertexIndex : u32) -> @builtin(position) vec4f {
            var pos = array(
              vec2f(-1.0, -1.0),
              vec2f(-1.0,  1.0),
              vec2f( 1.0, -1.0),
              vec2f(-1.0,  1.0),
              vec2f( 1.0, -1.0),
              vec2f( 1.0,  1.0));
            return vec4f(pos[VertexIndex], 0.0, 1.0);
        }
        `;
          let textureLoadStoreCoord = '';
          switch (rwTexture.dimension) {
            case '1d':
              textureLoadStoreCoord = 'textureCoord.x';
              break;
            case '2d':
              textureLoadStoreCoord =
              rwTexture.depthOrArrayLayers > 1 ? 'textureCoord, z' : 'textureCoord';
              break;
            case '3d':
              textureLoadStoreCoord = 'vec3u(textureCoord, z)';
              break;
          }
          const fragmentShader = `
        ${textureDeclaration}
        @fragment
        fn main(@builtin(position) fragCoord: vec4f) -> @location(0) vec4f {
          let textureCoord = vec2u(fragCoord.xy);

          for (var z = 0u; z < ${rwTexture.depthOrArrayLayers}; z++) {
            let initialValue = textureLoad(rwTexture, ${textureLoadStoreCoord});
            let outputValue = initialValue * 2;
            textureStore(rwTexture, ${textureLoadStoreCoord}, outputValue);
          }

          return vec4f(0.0, 1.0, 0.0, 1.0);
        }
        `;
          const renderPipeline = device.createRenderPipeline({
            layout: 'auto',
            vertex: {
              module: device.createShaderModule({
                code: vertexShader
              })
            },
            fragment: {
              module: device.createShaderModule({
                code: fragmentShader
              }),
              targets: [
              {
                format: 'rgba8unorm'
              }]

            },
            primitive: {
              topology: 'triangle-list'
            }
          });

          const bindGroup = device.createBindGroup({
            layout: renderPipeline.getBindGroupLayout(0),
            entries: [
            {
              binding: 0,
              resource: rwTexture.createView()
            }]

          });

          const placeholderColorTexture = this.createTextureTracked({
            size: [rwTexture.width, rwTexture.height, 1],
            usage: GPUTextureUsage.RENDER_ATTACHMENT,
            format: 'rgba8unorm'
          });

          const renderPassEncoder = commandEncoder.beginRenderPass({
            colorAttachments: [
            {
              view: placeholderColorTexture.createView(),
              loadOp: 'clear',
              clearValue: { r: 0, g: 0, b: 0, a: 0 },
              storeOp: 'store'
            }]

          });
          renderPassEncoder.setPipeline(renderPipeline);
          renderPassEncoder.setBindGroup(0, bindGroup);
          renderPassEncoder.draw(6);
          renderPassEncoder.end();
          break;
        }
      case 'compute':{
          let textureLoadCoord = '';
          let textureStoreCoord = '';
          switch (rwTexture.dimension) {
            case '1d':
              textureLoadCoord = 'dimension - 1u - invocationID.x';
              textureStoreCoord = 'invocationID.x';
              break;
            case '2d':
              textureLoadCoord =
              rwTexture.depthOrArrayLayers > 1 ?
              `vec2u(dimension.x - 1u - invocationID.x, dimension.y - 1u - invocationID.y),
                   textureNumLayers(rwTexture) - 1u - invocationID.z` :
              `vec2u(dimension.x - 1u - invocationID.x, dimension.y - 1u - invocationID.y)`;
              textureStoreCoord =
              rwTexture.depthOrArrayLayers > 1 ?
              'invocationID.xy, invocationID.z' :
              'invocationID.xy';
              break;
            case '3d':
              textureLoadCoord = `
              vec3u(dimension.x - 1u - invocationID.x, dimension.y - 1u - invocationID.y,
                    dimension.z - 1u - invocationID.z)`;
              textureStoreCoord = 'invocationID';
              break;
          }

          const computeShader = `
        ${textureDeclaration}
        @compute
        @workgroup_size(${rwTexture.width}, ${rwTexture.height}, ${rwTexture.depthOrArrayLayers})
        fn main(@builtin(local_invocation_id) invocationID: vec3u) {
          let dimension = textureDimensions(rwTexture);

          let initialValue = textureLoad(rwTexture, ${textureLoadCoord});
          textureBarrier();

          textureStore(rwTexture, ${textureStoreCoord}, initialValue);
        }`;

          const computePipeline = device.createComputePipeline({
            compute: {
              module: device.createShaderModule({
                code: computeShader
              })
            },
            layout: 'auto'
          });
          const bindGroup = device.createBindGroup({
            layout: computePipeline.getBindGroupLayout(0),
            entries: [
            {
              binding: 0,
              resource: rwTexture.createView()
            }]

          });
          const computePassEncoder = commandEncoder.beginComputePass();
          computePassEncoder.setPipeline(computePipeline);
          computePassEncoder.setBindGroup(0, bindGroup);
          computePassEncoder.dispatchWorkgroups(1);
          computePassEncoder.end();
          break;
        }
    }
  }
}

export const g = makeTestGroup(F);

g.test('basic').
desc(
  `The basic functionality tests for read-write storage textures. In the test we read data from
    the read-write storage texture, do transforms and write the data back to the read-write storage
    texture. textureBarrier() is also called in the tests using compute pipelines.`
).
params((u) =>
u.
combine('format', kColorTextureFormats).
filter((p) => kTextureFormatInfo[p.format].color?.readWriteStorage === true).
combine('shaderStage', kShaderStagesForReadWriteStorageTexture).
combine('textureDimension', kTextureDimensions).
combine('depthOrArrayLayers', [1, 2]).
unless((p) => p.textureDimension === '1d' && p.depthOrArrayLayers > 1)
).
beforeAllSubcases((t) => {
  t.skipIfTextureFormatNotUsableAsStorageTexture(t.params.format);
}).
fn((t) => {
  const { format, shaderStage, textureDimension, depthOrArrayLayers } = t.params;

  // In compatibility mode the lowest maxComputeInvocationsPerWorkgroup is 128 vs non-compat which is 256
  // So in non-compat we get 16 * 8 * 2, vs compat where we get 8 * 8 * 2
  const kWidth = t.isCompatibility ? 8 : 16;
  const height = textureDimension === '1d' ? 1 : 8;
  const textureSize = [kWidth, height, depthOrArrayLayers];
  const storageTexture = t.createTextureTracked({
    format,
    dimension: textureDimension,
    size: textureSize,
    usage: GPUTextureUsage.COPY_SRC | GPUTextureUsage.COPY_DST | GPUTextureUsage.STORAGE_BINDING
  });

  const bytesPerBlock = kTextureFormatInfo[format].bytesPerBlock;
  const initialData = t.GetInitialData(storageTexture);
  t.queue.writeTexture(
    { texture: storageTexture },
    initialData,
    {
      bytesPerRow: bytesPerBlock * kWidth,
      rowsPerImage: height
    },
    textureSize
  );

  const commandEncoder = t.device.createCommandEncoder();

  t.RecordCommandsToTransform(t.device, shaderStage, commandEncoder, storageTexture);

  const expectedData = t.GetExpectedData(shaderStage, storageTexture, initialData);
  const readbackBuffer = t.createBufferTracked({
    size: expectedData.byteLength,
    usage: GPUBufferUsage.COPY_SRC | GPUBufferUsage.COPY_DST
  });
  const bytesPerRow = align(bytesPerBlock * kWidth, 256);
  commandEncoder.copyTextureToBuffer(
    {
      texture: storageTexture
    },
    {
      buffer: readbackBuffer,
      bytesPerRow,
      rowsPerImage: height
    },
    textureSize
  );
  t.queue.submit([commandEncoder.finish()]);

  switch (format) {
    case 'r32sint':
      t.expectGPUBufferValuesEqual(readbackBuffer, new Int32Array(expectedData));
      break;
    case 'r32uint':
      t.expectGPUBufferValuesEqual(readbackBuffer, new Uint32Array(expectedData));
      break;
    case 'r32float':
      t.expectGPUBufferValuesEqual(readbackBuffer, new Float32Array(expectedData));
      break;
  }
});