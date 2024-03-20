/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/import { assert, unreachable } from '../../../../../common/util/util.js';import { kTextureFormatInfo } from '../../../../format_info.js';import { virtualMipSize } from '../../../../util/texture/base.js';
import {
  kTexelRepresentationInfo,
  getSingleDataType,
  getComponentReadbackTraits } from
'../../../../util/texture/texel_data.js';



export const checkContentsBySampling = (
t,
params,
texture,
state,
subresourceRange) =>
{
  assert(params.format in kTextureFormatInfo);
  const format = params.format;
  const rep = kTexelRepresentationInfo[format];

  for (const { level, layers } of subresourceRange.mipLevels()) {
    const [width, height, depth] = virtualMipSize(
      params.dimension,
      [t.textureWidth, t.textureHeight, t.textureDepth],
      level
    );

    const { ReadbackTypedArray, shaderType } = getComponentReadbackTraits(
      getSingleDataType(format)
    );

    const componentOrder = rep.componentOrder;
    const componentCount = componentOrder.length;

    // For single-component textures, generates .r
    // For multi-component textures, generates ex.)
    //  .rgba[i], .bgra[i], .rgb[i]
    const indexExpression =
    componentCount === 1 ?
    componentOrder[0].toLowerCase() :
    componentOrder.map((c) => c.toLowerCase()).join('') + '[i]';

    const viewDimension =
    t.isCompatibility && params.dimension === '2d' && texture.depthOrArrayLayers > 1 ?
    '2d-array' :
    params.dimension;
    const _xd = `_${viewDimension.replace('-', '_')}`;
    const _multisampled = params.sampleCount > 1 ? '_multisampled' : '';
    const texelIndexExpression =
    viewDimension === '2d' ?
    'vec2<i32>(GlobalInvocationID.xy)' :
    viewDimension === '2d-array' ?
    'vec2<i32>(GlobalInvocationID.xy), constants.layer' :
    viewDimension === '3d' ?
    'vec3<i32>(GlobalInvocationID.xyz)' :
    viewDimension === '1d' ?
    'i32(GlobalInvocationID.x)' :
    unreachable();
    const computePipeline = t.device.createComputePipeline({
      layout: 'auto',
      compute: {
        entryPoint: 'main',
        module: t.device.createShaderModule({
          code: `
            struct Constants {
              level : i32,
              layer : i32,
            };

            @group(0) @binding(0) var<uniform> constants : Constants;
            @group(0) @binding(1) var myTexture : texture${_multisampled}${_xd}<${shaderType}>;

            struct Result {
              values : array<${shaderType}>
            };
            @group(0) @binding(3) var<storage, read_write> result : Result;

            @compute @workgroup_size(1)
            fn main(@builtin(global_invocation_id) GlobalInvocationID : vec3<u32>) {
              let flatIndex : u32 = ${componentCount}u * (
                ${width}u * ${height}u * GlobalInvocationID.z +
                ${width}u * GlobalInvocationID.y +
                GlobalInvocationID.x
              );
              let texel : vec4<${shaderType}> = textureLoad(
                myTexture, ${texelIndexExpression}, constants.level);

              for (var i : u32 = 0u; i < ${componentCount}u; i = i + 1u) {
                result.values[flatIndex + i] = texel.${indexExpression};
              }
            }`
        })
      }
    });

    for (const layer of layers) {
      const ubo = t.device.createBuffer({
        mappedAtCreation: true,
        size: 8,
        usage: GPUBufferUsage.UNIFORM | GPUBufferUsage.COPY_DST
      });
      new Int32Array(ubo.getMappedRange()).set([level, layer]);
      ubo.unmap();

      const byteLength =
      width * height * depth * ReadbackTypedArray.BYTES_PER_ELEMENT * rep.componentOrder.length;
      const resultBuffer = t.device.createBuffer({
        size: byteLength,
        usage: GPUBufferUsage.STORAGE | GPUBufferUsage.COPY_SRC
      });
      t.trackForCleanup(resultBuffer);

      const viewDescriptor = {
        ...(!t.isCompatibility && {
          baseArrayLayer: layer,
          arrayLayerCount: 1
        }),
        dimension: viewDimension
      };

      const bindGroup = t.device.createBindGroup({
        layout: computePipeline.getBindGroupLayout(0),
        entries: [
        {
          binding: 0,
          resource: { buffer: ubo }
        },
        {
          binding: 1,
          resource: texture.createView(viewDescriptor)
        },
        {
          binding: 3,
          resource: {
            buffer: resultBuffer
          }
        }]

      });

      const commandEncoder = t.device.createCommandEncoder();
      const pass = commandEncoder.beginComputePass();
      pass.setPipeline(computePipeline);
      pass.setBindGroup(0, bindGroup);
      pass.dispatchWorkgroups(width, height, depth);
      pass.end();
      t.queue.submit([commandEncoder.finish()]);
      ubo.destroy();

      const expectedValues = new ReadbackTypedArray(new ArrayBuffer(byteLength));
      const expectedState = t.stateToTexelComponents[state];
      let i = 0;
      for (let d = 0; d < depth; ++d) {
        for (let h = 0; h < height; ++h) {
          for (let w = 0; w < width; ++w) {
            for (const c of rep.componentOrder) {
              const value = expectedState[c];
              assert(value !== undefined);
              expectedValues[i++] = value;
            }
          }
        }
      }
      t.expectGPUBufferValuesEqual(resultBuffer, expectedValues);
    }
  }
};