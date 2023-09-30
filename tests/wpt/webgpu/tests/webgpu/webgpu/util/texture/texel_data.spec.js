/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/ export const description =
  'Test helpers for texel data produce the expected data in the shader';
import { makeTestGroup } from '../../../common/framework/test_group.js';
import { assert } from '../../../common/util/util.js';
import { kEncodableTextureFormats, kTextureFormatInfo } from '../../format_info.js';
import { GPUTest } from '../../gpu_test.js';

import {
  kTexelRepresentationInfo,
  getSingleDataType,
  getComponentReadbackTraits,
} from './texel_data.js';

export const g = makeTestGroup(GPUTest);

function doTest(t) {
  const { format } = t.params;
  const componentData = t.params.componentData;

  const rep = kTexelRepresentationInfo[format];
  const texelData = rep.pack(componentData);
  const texture = t.device.createTexture({
    format,
    size: [1, 1, 1],
    usage: GPUTextureUsage.COPY_DST | GPUTextureUsage.TEXTURE_BINDING,
  });

  t.device.queue.writeTexture(
    { texture },
    texelData,
    {
      bytesPerRow: texelData.byteLength,
    },
    [1]
  );

  const { ReadbackTypedArray, shaderType } = getComponentReadbackTraits(getSingleDataType(format));

  const shader = `
  @group(0) @binding(0) var tex : texture_2d<${shaderType}>;

  struct Output {
    ${rep.componentOrder.map(C => `result${C} : ${shaderType},`).join('\n')}
  };
  @group(0) @binding(1) var<storage, read_write> output : Output;

  @compute @workgroup_size(1)
  fn main() {
      var texel : vec4<${shaderType}> = textureLoad(tex, vec2<i32>(0, 0), 0);
      ${rep.componentOrder.map(C => `output.result${C} = texel.${C.toLowerCase()};`).join('\n')}
      return;
  }`;

  const pipeline = t.device.createComputePipeline({
    layout: 'auto',
    compute: {
      module: t.device.createShaderModule({
        code: shader,
      }),
      entryPoint: 'main',
    },
  });

  const outputBuffer = t.device.createBuffer({
    size: rep.componentOrder.length * 4,
    usage: GPUBufferUsage.STORAGE | GPUBufferUsage.COPY_SRC,
  });

  const bindGroup = t.device.createBindGroup({
    layout: pipeline.getBindGroupLayout(0),
    entries: [
      {
        binding: 0,
        resource: texture.createView(),
      },
      {
        binding: 1,
        resource: {
          buffer: outputBuffer,
        },
      },
    ],
  });

  const encoder = t.device.createCommandEncoder();
  const pass = encoder.beginComputePass();
  pass.setPipeline(pipeline);
  pass.setBindGroup(0, bindGroup);
  pass.dispatchWorkgroups(1);
  pass.end();
  t.device.queue.submit([encoder.finish()]);

  t.expectGPUBufferValuesEqual(
    outputBuffer,
    new ReadbackTypedArray(
      rep.componentOrder.map(c => {
        const value = rep.decode(componentData)[c];
        assert(value !== undefined);
        return value;
      })
    )
  );
}

// Make a test parameter by mapping a format and each component to a texel component
// data value.
function makeParam(format, fn) {
  const rep = kTexelRepresentationInfo[format];
  return {
    R: rep.componentInfo.R ? fn(rep.componentInfo.R.bitLength, 0) : undefined,
    G: rep.componentInfo.G ? fn(rep.componentInfo.G.bitLength, 1) : undefined,
    B: rep.componentInfo.B ? fn(rep.componentInfo.B.bitLength, 2) : undefined,
    A: rep.componentInfo.A ? fn(rep.componentInfo.A.bitLength, 3) : undefined,
  };
}

g.test('unorm_texel_data_in_shader')
  .params(u =>
    u
      .combine('format', kEncodableTextureFormats)
      .filter(({ format }) => {
        const info = kTextureFormatInfo[format];
        return !!info.color && info.color.copyDst && getSingleDataType(format) === 'unorm';
      })
      .beginSubcases()
      .expand('componentData', ({ format }) => {
        const max = bitLength => Math.pow(2, bitLength) - 1;
        return [
          // Test extrema
          makeParam(format, () => 0),
          makeParam(format, bitLength => max(bitLength)),

          // Test a middle value
          makeParam(format, bitLength => Math.floor(max(bitLength) / 2)),

          // Test mixed values
          makeParam(format, (bitLength, i) => {
            const offset = [0.13, 0.63, 0.42, 0.89];
            return Math.floor(offset[i] * max(bitLength));
          }),
        ];
      })
  )
  .beforeAllSubcases(t => {
    t.skipIfTextureFormatNotSupported(t.params.format);
  })
  .fn(doTest);

g.test('snorm_texel_data_in_shader')
  .params(u =>
    u
      .combine('format', kEncodableTextureFormats)
      .filter(({ format }) => {
        const info = kTextureFormatInfo[format];
        return !!info.color && info.color.copyDst && getSingleDataType(format) === 'snorm';
      })
      .beginSubcases()
      .expand('componentData', ({ format }) => {
        const max = bitLength => Math.pow(2, bitLength - 1) - 1;
        return [
          // Test extrema
          makeParam(format, () => 0),
          makeParam(format, bitLength => max(bitLength)),
          makeParam(format, bitLength => -max(bitLength)),
          makeParam(format, bitLength => -max(bitLength) - 1),

          // Test a middle value
          makeParam(format, bitLength => Math.floor(max(bitLength) / 2)),

          // Test mixed values
          makeParam(format, (bitLength, i) => {
            const offset = [0.13, 0.63, 0.42, 0.89];
            const range = 2 * max(bitLength);
            return -max(bitLength) + Math.floor(offset[i] * range);
          }),
        ];
      })
  )
  .fn(doTest);

g.test('uint_texel_data_in_shader')
  .params(u =>
    u
      .combine('format', kEncodableTextureFormats)
      .filter(({ format }) => {
        const info = kTextureFormatInfo[format];
        return !!info.color && info.color.copyDst && getSingleDataType(format) === 'uint';
      })
      .beginSubcases()
      .expand('componentData', ({ format }) => {
        const max = bitLength => Math.pow(2, bitLength) - 1;
        return [
          // Test extrema
          makeParam(format, () => 0),
          makeParam(format, bitLength => max(bitLength)),

          // Test a middle value
          makeParam(format, bitLength => Math.floor(max(bitLength) / 2)),

          // Test mixed values
          makeParam(format, (bitLength, i) => {
            const offset = [0.13, 0.63, 0.42, 0.89];
            return Math.floor(offset[i] * max(bitLength));
          }),
        ];
      })
  )
  .fn(doTest);

g.test('sint_texel_data_in_shader')
  .params(u =>
    u
      .combine('format', kEncodableTextureFormats)
      .filter(({ format }) => {
        const info = kTextureFormatInfo[format];
        return !!info.color && info.color.copyDst && getSingleDataType(format) === 'sint';
      })
      .beginSubcases()
      .expand('componentData', ({ format }) => {
        const max = bitLength => Math.pow(2, bitLength - 1) - 1;
        return [
          // Test extrema
          makeParam(format, () => 0),
          makeParam(format, bitLength => max(bitLength)),
          makeParam(format, bitLength => -max(bitLength) - 1),

          // Test a middle value
          makeParam(format, bitLength => Math.floor(max(bitLength) / 2)),

          // Test mixed values
          makeParam(format, (bitLength, i) => {
            const offset = [0.13, 0.63, 0.42, 0.89];
            const range = 2 * max(bitLength);
            return -max(bitLength) + Math.floor(offset[i] * range);
          }),
        ];
      })
  )
  .fn(doTest);

g.test('float_texel_data_in_shader')
  .desc(
    `
TODO: Test NaN, Infinity, -Infinity [1]`
  )
  .params(u =>
    u
      .combine('format', kEncodableTextureFormats)
      .filter(({ format }) => {
        const info = kTextureFormatInfo[format];
        return !!info.color && info.color.copyDst && getSingleDataType(format) === 'float';
      })
      .beginSubcases()
      .expand('componentData', ({ format }) => {
        return [
          // Test extrema
          makeParam(format, () => 0),

          // [1]: Test NaN, Infinity, -Infinity

          // Test some values
          makeParam(format, () => 0.1199951171875),
          makeParam(format, () => 1.4072265625),
          makeParam(format, () => 24928),
          makeParam(format, () => -0.1319580078125),
          makeParam(format, () => -323.25),
          makeParam(format, () => -7440),

          // Test mixed values
          makeParam(format, (bitLength, i) => {
            return [24896, -0.1319580078125, -323.25, -234.375][i];
          }),
        ];
      })
  )
  .fn(doTest);

g.test('ufloat_texel_data_in_shader')
  .desc(
    `
TODO: Test NaN, Infinity [1]`
  )
  .params(u =>
    u
      .combine('format', kEncodableTextureFormats)
      .filter(({ format }) => {
        const info = kTextureFormatInfo[format];
        return !!info.color && info.color.copyDst && getSingleDataType(format) === 'ufloat';
      })
      .beginSubcases()
      .expand('componentData', ({ format }) => {
        return [
          // Test extrema
          makeParam(format, () => 0),

          // [2]: Test NaN, Infinity

          // Test some values
          makeParam(format, () => 0.119140625),
          makeParam(format, () => 1.40625),
          makeParam(format, () => 24896),

          // Test scattered mixed values
          makeParam(format, (bitLength, i) => {
            return [24896, 1.40625, 0.119140625, 0.23095703125][i];
          }),

          // Test mixed values that are close in magnitude.
          makeParam(format, (bitLength, i) => {
            return [0.1337890625, 0.17919921875, 0.119140625, 0.125][i];
          }),
        ];
      })
  )
  .fn(doTest);
