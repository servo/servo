/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Tests for the behavior of read-only storage textures.

TODO:
- Test mipmap level > 0
- Test resource usage transitions with read-only storage textures
`;import { makeTestGroup } from '../../../../common/framework/test_group.js';
import { unreachable, assert } from '../../../../common/util/util.js';
import { Float16Array } from '../../../../external/petamoriken/float16/float16.js';
import { kTextureDimensions } from '../../../capability_info.js';
import {

  getBlockInfoForColorTextureFormat,
  getTextureFormatType,
  kPossibleStorageTextureFormats } from
'../../../format_info.js';
import { AllFeaturesMaxLimitsGPUTest } from '../../../gpu_test.js';
import { kValidShaderStages } from '../../../util/shader.js';

function getComponentCountForFormat(format) {
  switch (format) {
    case 'r8unorm':
    case 'r8uint':
    case 'r8snorm':
    case 'r8sint':
    case 'r16unorm':
    case 'r16uint':
    case 'r16snorm':
    case 'r16sint':
    case 'r16float':
    case 'r32float':
    case 'r32sint':
    case 'r32uint':
      return 1;
    case 'rg8unorm':
    case 'rg8uint':
    case 'rg8snorm':
    case 'rg8sint':
    case 'rg16unorm':
    case 'rg16uint':
    case 'rg16snorm':
    case 'rg16sint':
    case 'rg16float':
    case 'rg32float':
    case 'rg32sint':
    case 'rg32uint':
      return 2;
    case 'rg11b10ufloat':
      return 3;
    case 'rgba32float':
    case 'rgba32sint':
    case 'rgba32uint':
    case 'rgba8sint':
    case 'rgba8uint':
    case 'rgba8snorm':
    case 'rgba8unorm':
    case 'rgba16float':
    case 'rgba16sint':
    case 'rgba16uint':
    case 'bgra8unorm':
    case 'rgba16unorm':
    case 'rgba16snorm':
    case 'rgb10a2uint':
    case 'rgb10a2unorm':
      return 4;
    default:
      unreachable();
      return 0;
  }
}

class F extends AllFeaturesMaxLimitsGPUTest {
  initTextureAndGetExpectedOutputBufferData(
  storageTexture,
  format)
  {
    const { bytesPerBlock } = getBlockInfoForColorTextureFormat(format);

    const width = storageTexture.width;
    const height = storageTexture.height;
    const depthOrArrayLayers = storageTexture.depthOrArrayLayers;

    const texelData = new ArrayBuffer(bytesPerBlock * width * height * depthOrArrayLayers);
    const texelTypedDataView = this.getTypedArrayBufferViewForTexelData(texelData, format);
    const componentCount = getComponentCountForFormat(format);
    const outputBufferData = new ArrayBuffer(4 * 4 * width * height * depthOrArrayLayers);
    const outputBufferTypedData = this.getTypedArrayBufferForOutputBufferData(
      outputBufferData,
      format
    );

    const setData = (
    texelValue,
    outputValue,
    texelDataIndex,
    component,
    outputComponent = component) =>
    {
      const texelComponentIndex = texelDataIndex * componentCount + component;
      texelTypedDataView[texelComponentIndex] = texelValue;
      const outputTexelComponentIndex = texelDataIndex * 4 + outputComponent;
      outputBufferTypedData[outputTexelComponentIndex] = outputValue;
    };
    for (let z = 0; z < depthOrArrayLayers; ++z) {
      for (let y = 0; y < height; ++y) {
        for (let x = 0; x < width; ++x) {
          const texelDataIndex = z * width * height + y * width + x;
          outputBufferTypedData[4 * texelDataIndex] = 0;
          outputBufferTypedData[4 * texelDataIndex + 1] = 0;
          outputBufferTypedData[4 * texelDataIndex + 2] = 0;
          outputBufferTypedData[4 * texelDataIndex + 3] = 1;
          // Packed formats like rgb10a2unorm, rg11b10ufloat, and rgb10a2uint store multiple color components within a single 32-bit integer.
          // This means their TypedArray uses a single element per texel, so they are handled separately from other formats
          if (format === 'rgb10a2unorm') {
            const texelValue = 4 * texelDataIndex + 1;
            const r = texelValue % 1024;
            const g = texelValue * 2 % 1024;
            const b = texelValue * 3 % 1024;
            const a = 3;
            const packedValue = a << 30 | b << 20 | g << 10 | r;
            const texelComponentIndex = texelDataIndex;
            texelTypedDataView[texelComponentIndex] = packedValue;
            outputBufferTypedData[texelDataIndex * 4] = r / 1023.0;
            outputBufferTypedData[texelDataIndex * 4 + 1] = g / 1023.0;
            outputBufferTypedData[texelDataIndex * 4 + 2] = b / 1023.0;
            outputBufferTypedData[texelDataIndex * 4 + 3] = a / 3.0;
          } else if (format === 'rg11b10ufloat') {
            const kFloat11One = 0x3c0;
            const kFloat10Zero = 0;
            const r = kFloat11One;
            const g = kFloat11One;
            const b = kFloat10Zero;
            const packedValue = b << 22 | g << 11 | r;
            const texelComponentIndex = texelDataIndex;
            texelTypedDataView[texelComponentIndex] = packedValue;
            outputBufferTypedData[texelDataIndex * 4] = 1.0;
            outputBufferTypedData[texelDataIndex * 4 + 1] = 1.0;
            outputBufferTypedData[texelDataIndex * 4 + 2] = 0;
          } else if (format === 'rgb10a2uint') {
            const texelValue = 4 * texelDataIndex + 1;
            const r = texelValue % 1024;
            const g = texelValue * 2 % 1024;
            const b = texelValue * 3 % 1024;
            const a = 3;
            const packedValue = a << 30 | b << 20 | g << 10 | r;
            const texelComponentIndex = texelDataIndex;
            texelTypedDataView[texelComponentIndex] = packedValue;
            outputBufferTypedData[texelDataIndex * 4] = r;
            outputBufferTypedData[texelDataIndex * 4 + 1] = g;
            outputBufferTypedData[texelDataIndex * 4 + 2] = b;
            outputBufferTypedData[texelDataIndex * 4 + 3] = a;
          } else {
            for (let component = 0; component < componentCount; ++component) {
              switch (format) {
                case 'r32uint':
                case 'rg32uint':
                case 'rgba16uint':
                case 'rgba32uint':
                case 'r8uint':
                case 'rg8uint':
                case 'rgba8uint':{
                    const texelValue = (4 * texelDataIndex + component + 1) % 256;
                    setData(texelValue, texelValue, texelDataIndex, component);
                    break;
                  }
                case 'r16uint':
                case 'rg16uint':{
                    const texelValue = (4 * texelDataIndex + component + 1) % 65536;
                    setData(texelValue, texelValue, texelDataIndex, component);
                    break;
                  }
                case 'r8unorm':
                case 'rg8unorm':
                case 'rgba8unorm':{
                    const texelValue = (4 * texelDataIndex + component + 1) % 256;
                    const outputValue = texelValue / 255.0;
                    setData(texelValue, outputValue, texelDataIndex, component);
                    break;
                  }
                case 'bgra8unorm':{
                    const texelValue = (4 * texelDataIndex + component + 1) % 256;
                    const outputValue = texelValue / 255.0;
                    // BGRA -> RGBA
                    assert(component < 4);
                    const outputComponent = [2, 1, 0, 3][component];
                    setData(texelValue, outputValue, texelDataIndex, component, outputComponent);
                    break;
                  }
                case 'r16unorm':
                case 'rg16unorm':
                case 'rgba16unorm':{
                    const texelValue = (4 * texelDataIndex + component + 1) % 65536;
                    const outputValue = texelValue / 65535.0;
                    setData(texelValue, outputValue, texelDataIndex, component);
                    break;
                  }
                case 'r32sint':
                case 'rg32sint':
                case 'rgba16sint':
                case 'rgba32sint':{
                    const texelValue =
                    (texelDataIndex & 1 ? 1 : -1) * (4 * texelDataIndex + component + 1);
                    setData(texelValue, texelValue, texelDataIndex, component);
                    break;
                  }
                case 'r8sint':
                case 'rg8sint':
                case 'rgba8sint':{
                    const texelValue = (4 * texelDataIndex + component + 1) % 256 - 128;
                    setData(texelValue, texelValue, texelDataIndex, component);
                    break;
                  }
                case 'r16sint':
                case 'rg16sint':{
                    const signedValue =
                    (texelDataIndex & 1 ? 1 : -1) * (
                    (4 * texelDataIndex + component + 1) % 65536 - 32768);
                    setData(signedValue, signedValue, texelDataIndex, component);
                    break;
                  }
                case 'r8snorm':
                case 'rg8snorm':
                case 'rgba8snorm':{
                    const texelValue = (4 * texelDataIndex + component + 1) % 256 - 128;
                    const outputValue = Math.max(texelValue / 127.0, -1.0);
                    setData(texelValue, outputValue, texelDataIndex, component);
                    break;
                  }
                case 'r16snorm':
                case 'rg16snorm':
                case 'rgba16snorm':{
                    const texelValue = (4 * texelDataIndex + component + 1) % 65536 - 32768;
                    const outputValue = Math.max(texelValue / 32767.0, -1.0);
                    setData(texelValue, outputValue, texelDataIndex, component);
                    break;
                  }
                case 'r32float':
                case 'rg32float':
                case 'rgba32float':{
                    const texelValue = (4 * texelDataIndex + component + 1) / 10.0;
                    setData(texelValue, texelValue, texelDataIndex, component);
                    break;
                  }
                case 'r16float':
                case 'rg16float':
                case 'rgba16float':{
                    const texelValue = (4 * texelDataIndex + component + 1) / 10.0;
                    const f16Array = new Float16Array(1);
                    f16Array[0] = texelValue;
                    setData(texelValue, f16Array[0], texelDataIndex, component);
                    break;
                  }
                default:
                  unreachable();
                  break;
              }
            }
          }
        }
      }
    }
    this.queue.writeTexture(
      {
        texture: storageTexture
      },
      texelData,
      {
        bytesPerRow: bytesPerBlock * width,
        rowsPerImage: height
      },
      [width, height, depthOrArrayLayers]
    );

    return outputBufferData;
  }

  getTypedArrayBufferForOutputBufferData(arrayBuffer, format) {
    switch (getTextureFormatType(format)) {
      case 'uint':
        return new Uint32Array(arrayBuffer);
      case 'sint':
        return new Int32Array(arrayBuffer);
      case 'float':
      case 'unfilterable-float':
        return new Float32Array(arrayBuffer);
      default:
        unreachable();
    }
  }

  getTypedArrayBufferViewForTexelData(arrayBuffer, format) {
    switch (format) {
      case 'r32uint':
      case 'rg32uint':
      case 'rgba32uint':
      case 'rgb10a2uint':
      case 'rgb10a2unorm':
        return new Uint32Array(arrayBuffer);
      case 'rgba8uint':
      case 'rgba8unorm':
      case 'bgra8unorm':
      case 'r8unorm':
      case 'r8uint':
      case 'rg8unorm':
      case 'rg8uint':
        return new Uint8Array(arrayBuffer);
      case 'rgba16uint':
      case 'r16unorm':
      case 'rg16unorm':
      case 'rgba16unorm':
      case 'r16uint':
      case 'rg16uint':
        return new Uint16Array(arrayBuffer);
      case 'r32sint':
      case 'rg32sint':
      case 'rgba32sint':
      case 'rg11b10ufloat':
        return new Int32Array(arrayBuffer);
      case 'rgba8sint':
      case 'rgba8snorm':
      case 'r8snorm':
      case 'r8sint':
      case 'rg8snorm':
      case 'rg8sint':
        return new Int8Array(arrayBuffer);
      case 'rgba16sint':
      case 'r16snorm':
      case 'rg16snorm':
      case 'rgba16snorm':
      case 'r16sint':
      case 'rg16sint':
        return new Int16Array(arrayBuffer);
      case 'r32float':
      case 'rg32float':
      case 'rgba32float':
        return new Float32Array(arrayBuffer);
      case 'r16float':
      case 'rg16float':
      case 'rgba16float':
        return new Float16Array(arrayBuffer);
      default:
        unreachable();
        return new Uint8Array(arrayBuffer);
    }
  }

  getOutputBufferWGSLType(format) {
    switch (getTextureFormatType(format)) {
      case 'uint':
        return 'vec4u';
      case 'sint':
        return 'vec4i';
      case 'float':
      case 'unfilterable-float':
        return 'vec4f';
      default:
        unreachable();
        return '';
    }
  }

  doTransform(
  storageTexture,
  shaderStage,
  format,
  outputBuffer)
  {
    let declaration = '';
    switch (storageTexture.dimension) {
      case '1d':
        declaration = 'texture_storage_1d';
        break;
      case '2d':
        declaration =
        storageTexture.depthOrArrayLayers > 1 ? 'texture_storage_2d_array' : 'texture_storage_2d';
        break;
      case '3d':
        declaration = 'texture_storage_3d';
        break;
    }
    const textureDeclaration = `
    @group(0) @binding(0) var readOnlyTexture: ${declaration}<${format}, read>;
    `;

    const bindGroupEntries = [
    {
      binding: 0,
      resource: storageTexture.createView()
    },
    ...(shaderStage === 'compute' ?
    [
    {
      binding: 1,
      resource: {
        buffer: outputBuffer
      }
    }] :

    [])];


    const commandEncoder = this.device.createCommandEncoder();

    switch (shaderStage) {
      case 'compute':{
          let textureLoadCoord = '';
          switch (storageTexture.dimension) {
            case '1d':
              textureLoadCoord = 'invocationID.x';
              break;
            case '2d':
              textureLoadCoord =
              storageTexture.depthOrArrayLayers > 1 ?
              `vec2u(invocationID.x, invocationID.y), invocationID.z` :
              `vec2u(invocationID.x, invocationID.y)`;
              break;
            case '3d':
              textureLoadCoord = 'invocationID';
              break;
          }

          const computeShader = `
      ${textureDeclaration}
      @group(0) @binding(1)
      var<storage,read_write> outputBuffer : array<${this.getOutputBufferWGSLType(format)}>;

      @compute
      @workgroup_size(
        ${storageTexture.width}, ${storageTexture.height}, ${storageTexture.depthOrArrayLayers})
      fn main(
        @builtin(local_invocation_id) invocationID: vec3u,
        @builtin(local_invocation_index) invocationIndex: u32) {
        let initialValue = textureLoad(readOnlyTexture, ${textureLoadCoord});
        outputBuffer[invocationIndex] = initialValue;
      }`;
          const computePipeline = this.device.createComputePipeline({
            compute: {
              module: this.device.createShaderModule({
                code: computeShader
              })
            },
            layout: 'auto'
          });
          const bindGroup = this.device.createBindGroup({
            layout: computePipeline.getBindGroupLayout(0),
            entries: bindGroupEntries
          });

          const computePassEncoder = commandEncoder.beginComputePass();
          computePassEncoder.setPipeline(computePipeline);
          computePassEncoder.setBindGroup(0, bindGroup);
          computePassEncoder.dispatchWorkgroups(1);
          computePassEncoder.end();
          break;
        }
      case 'fragment':{
          let textureLoadCoord = '';
          switch (storageTexture.dimension) {
            case '1d':
              textureLoadCoord = 'textureCoord.x';
              break;
            case '2d':
              textureLoadCoord =
              storageTexture.depthOrArrayLayers > 1 ? 'textureCoord, coordZ' : 'textureCoord';
              break;
            case '3d':
              textureLoadCoord = 'vec3u(textureCoord, coordZ)';
              break;
          }

          const shader = `
        ${textureDeclaration}
        @fragment
        fn fs(@builtin(position) fragCoord: vec4f) -> @location(0) vec4u {
          let coordX = u32(fragCoord.x);
          let coordY = u32(fragCoord.y) % ${storageTexture.height}u;
          let coordZ = u32(fragCoord.y) / ${storageTexture.height}u;
          let textureCoord = vec2u(coordX, coordY);
          return bitcast<vec4u>(textureLoad(readOnlyTexture, ${textureLoadCoord}));
        }

        @vertex
        fn vs(@builtin(vertex_index) vertexIndex : u32) -> @builtin(position) vec4f {
            var pos = array(
              vec2f(-1.0,  3.0),
              vec2f( 3.0, -1.0),
              vec2f(-1.0, -1.0));
            return vec4f(pos[vertexIndex], 0.0, 1.0);
        }
        `;

          const module = this.device.createShaderModule({
            code: shader
          });
          const renderPipeline = this.device.createRenderPipeline({
            layout: 'auto',
            vertex: { module },
            fragment: { module, targets: [{ format: 'rgba32uint' }] },
            primitive: { topology: 'triangle-list' }
          });

          const bindGroup = this.device.createBindGroup({
            layout: renderPipeline.getBindGroupLayout(0),
            entries: bindGroupEntries
          });

          // This is just so our buffer compare is the same as the compute stage.
          // Otherwise, we'd have to pad every row to a multiple of 256 bytes and
          // change the comparison code to take that into account.
          assert(storageTexture.width === 16, `width must be 16 because we require 256 bytesPerRow`);
          const placeholderColorTexture = this.createTextureTracked({
            size: [storageTexture.width, storageTexture.height * storageTexture.depthOrArrayLayers],
            usage: GPUTextureUsage.RENDER_ATTACHMENT | GPUTextureUsage.COPY_SRC,
            format: 'rgba32uint'
          });

          const renderPassEncoder = commandEncoder.beginRenderPass({
            colorAttachments: [
            {
              view: placeholderColorTexture.createView(),
              loadOp: 'clear',
              storeOp: 'store'
            }]

          });
          renderPassEncoder.setPipeline(renderPipeline);
          renderPassEncoder.setBindGroup(0, bindGroup);
          renderPassEncoder.draw(3);
          renderPassEncoder.end();

          commandEncoder.copyTextureToBuffer(
            { texture: placeholderColorTexture },
            {
              buffer: outputBuffer,
              bytesPerRow: 256
            },
            placeholderColorTexture
          );
          break;
        }
      case 'vertex':{
          // We draw storageTexture.Width by (storageTexture.height * storageTexture.depthOrArrayLayers)
          // points via 'point-list' to a placeholderColorTexture of the same size.
          //
          // We use the @builtin(vertex_index) to compute a coord in the source texture
          // and use that same coord to compute a place to render in the point in the placeholder.
          let loadFromTextureWGSL = '';
          switch (storageTexture.dimension) {
            case '1d':
              loadFromTextureWGSL = `
              output.vertex_out = textureLoad(readOnlyTexture, coordX);`;
              break;
            case '2d':
              if (storageTexture.depthOrArrayLayers === 1) {
                loadFromTextureWGSL = `
                output.vertex_out = textureLoad(readOnlyTexture, vec2u(coordX, coordY));`;
              } else {
                loadFromTextureWGSL = loadFromTextureWGSL.concat(`
                output.vertex_out =
                  textureLoad(readOnlyTexture, vec2u(coordX, coordY), coordZ);`);
              }
              break;
            case '3d':
              loadFromTextureWGSL = loadFromTextureWGSL.concat(`
              output.vertex_out = textureLoad(readOnlyTexture, vec3u(coordX, coordY, coordZ));`);
              break;
          }

          let outputToBufferWGSL = '';
          for (let layer = 0; layer < storageTexture.depthOrArrayLayers; ++layer) {
            outputToBufferWGSL = outputToBufferWGSL.concat(
              `
            let outputIndex${layer} =
              storageTextureTexelCountPerImage * ${layer}u +
              fragmentInput.tex_coord.y * ${storageTexture.width}u + fragmentInput.tex_coord.x;
            outputBuffer[outputIndex${layer}] = fragmentInput.vertex_out${layer};`
            );
          }

          const shader = `
        ${textureDeclaration}
        struct VertexOutput {
          @builtin(position) my_pos: vec4f,
          @location(0) @interpolate(flat, either)
            vertex_out: ${this.getOutputBufferWGSLType(format)},
        }
        @vertex
        fn vs_main(@builtin(vertex_index) vertexIndex : u32) -> VertexOutput {
            var output : VertexOutput;
            let coordX = vertexIndex % ${storageTexture.width}u;
            let coordY = vertexIndex / ${storageTexture.width}u % ${storageTexture.height}u;
            let coordZ = vertexIndex / ${storageTexture.width * storageTexture.height}u;
            let writePos = vec2f(f32(coordX), f32(coordY + coordZ * ${storageTexture.height}));
            let destSize = vec2f(
              ${storageTexture.width},
              ${storageTexture.height * storageTexture.depthOrArrayLayers});
            output.my_pos = vec4f((((writePos + 0.5) / destSize) * 2.0 - 1.0) * vec2f(1, -1), 0.0, 1.0);
            ${loadFromTextureWGSL}
            return output;
        }
        @fragment
        fn fs_main(fragmentInput : VertexOutput) -> @location(0) vec4u {
          let v = fragmentInput.vertex_out;
          return bitcast<vec4u>(v);
        }
        `;

          const module = this.device.createShaderModule({ code: shader });
          const renderPipeline = this.device.createRenderPipeline({
            layout: 'auto',
            vertex: { module },
            fragment: { module, targets: [{ format: 'rgba32uint' }] },
            primitive: { topology: 'point-list' }
          });

          const bindGroup = this.device.createBindGroup({
            layout: renderPipeline.getBindGroupLayout(0),
            entries: bindGroupEntries
          });

          const placeholderColorTexture = this.createTextureTracked({
            size: [storageTexture.width, storageTexture.height * storageTexture.depthOrArrayLayers],
            usage: GPUTextureUsage.RENDER_ATTACHMENT | GPUTextureUsage.COPY_SRC,
            format: 'rgba32uint'
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
          const texelCount =
          storageTexture.width * storageTexture.height * storageTexture.depthOrArrayLayers;
          renderPassEncoder.draw(texelCount);
          renderPassEncoder.end();

          commandEncoder.copyTextureToBuffer(
            { texture: placeholderColorTexture },
            {
              buffer: outputBuffer,
              bytesPerRow: 256
            },
            placeholderColorTexture
          );
          break;
        }
    }

    this.queue.submit([commandEncoder.finish()]);
  }
}

export const g = makeTestGroup(F);

g.test('basic').
desc(
  `The basic functionality tests for read-only storage textures. In the test we read data from
    the read-only storage texture, write the data into an output storage buffer, and check if the
    data in the output storage buffer is exactly what we expect.`
).
params((u) =>
u.
combine('format', kPossibleStorageTextureFormats).
combine('shaderStage', kValidShaderStages).
combine('dimension', kTextureDimensions).
combine('depthOrArrayLayers', [1, 2]).
unless((p) => p.dimension === '1d' && p.depthOrArrayLayers > 1)
).
fn((t) => {
  const { format, shaderStage, dimension, depthOrArrayLayers } = t.params;
  t.skipIfTextureFormatNotSupported(format);
  t.skipIfTextureFormatNotUsableWithStorageAccessMode('read-only', format);

  if (t.isCompatibility) {
    if (shaderStage === 'fragment') {
      t.skipIf(
        !(t.device.limits.maxStorageTexturesInFragmentStage > 0),
        `maxStorageTexturesInFragmentStage(${t.device.limits.
        maxStorageTexturesInFragmentStage}) is not > 0`
      );
    } else if (shaderStage === 'vertex') {
      t.skipIf(
        !(t.device.limits.maxStorageTexturesInVertexStage > 0),
        `maxStorageTexturesInVertexStage(${t.device.limits.
        maxStorageTexturesInVertexStage}) is not > 0`
      );
    }
  }

  const kWidth = 16;
  const height = dimension === '1d' ? 1 : 8;
  const storageTexture = t.createTextureTracked({
    format,
    dimension,
    size: [kWidth, height, depthOrArrayLayers],
    usage: GPUTextureUsage.COPY_SRC | GPUTextureUsage.COPY_DST | GPUTextureUsage.STORAGE_BINDING
  });

  const expectedData = t.initTextureAndGetExpectedOutputBufferData(storageTexture, format);

  const bytesPerRow = 4 * 4 * kWidth;
  assert(bytesPerRow === 256, 'bytesPerRow === 256');
  const outputBuffer = t.createBufferTracked({
    size: bytesPerRow * height * depthOrArrayLayers,
    usage: GPUBufferUsage.COPY_SRC | GPUBufferUsage.STORAGE | GPUBufferUsage.COPY_DST
  });

  t.doTransform(storageTexture, shaderStage, format, outputBuffer);

  switch (getTextureFormatType(format)) {
    case 'uint':
      t.expectGPUBufferValuesEqual(outputBuffer, new Uint32Array(expectedData));
      break;
    case 'sint':
      t.expectGPUBufferValuesEqual(outputBuffer, new Int32Array(expectedData));
      break;
    case 'float':
    case 'unfilterable-float':
      t.expectGPUBufferValuesEqual(outputBuffer, new Float32Array(expectedData));
      break;
    default:
      unreachable();
      break;
  }
});