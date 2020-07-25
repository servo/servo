/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/ export const description = 'Test uninitialized textures are initialized to zero when sampled.';
import { makeTestGroup } from '../../../../common/framework/test_group.js';
import { assert } from '../../../../common/framework/util/util.js';

import { getTexelDataRepresentation } from '../../../util/texture/texelData.js';

import {
  ReadMethod,
  TextureZeroInitTest,
  initializedStateAsFloat,
  initializedStateAsSint,
  initializedStateAsUint,
} from './texture_zero_init_test.js';

class SampledTextureClearTest extends TextureZeroInitTest {
  getSamplingReadbackPipeline(prefix, sampleCount, dimension) {
    const componentOrder = getTexelDataRepresentation(this.params.format).componentOrder;
    const MS = sampleCount > 1 ? 'MS' : '';
    const XD = dimension.toUpperCase();
    const componentCount = componentOrder.length;
    const indexExpression =
      componentCount === 1
        ? componentOrder[0].toLowerCase()
        : componentOrder.map(c => c.toLowerCase()).join('') + '[i]';

    const glsl = `#version 310 es
      precision highp float;
      precision highp ${prefix}texture${XD}${MS};
      precision highp sampler;

      layout(set = 0, binding = 0, std140) uniform Constants {
        int level;
      };

      layout(set = 0, binding = 1) uniform ${prefix}texture${XD}${MS} myTexture;
      layout(set = 0, binding = 2) uniform sampler mySampler;
      layout(set = 0, binding = 3, std430) writeonly buffer Result {
        uint result[];
      };

      void writeResult(uint flatIndex, uvec4 texel) {
        for (uint i = flatIndex; i < flatIndex + ${componentCount}u; ++i) {
          result[i] = texel.${indexExpression};
        }
      }

      void writeResult(uint flatIndex, ivec4 texel) {
        for (uint i = flatIndex; i < flatIndex + ${componentCount}u; ++i) {
          result[i] = uint(texel.${indexExpression});
        }
      }

      void writeResult(uint flatIndex, vec4 texel) {
        for (uint i = flatIndex; i < flatIndex + ${componentCount}u; ++i) {
          result[i] = floatBitsToUint(texel.${indexExpression});
        }
      }

      void main() {
        uint flatIndex = gl_NumWorkGroups.x * gl_GlobalInvocationID.y + gl_GlobalInvocationID.x;
        flatIndex = flatIndex * ${componentCount}u;

        writeResult(flatIndex, texelFetch(
          ${prefix}sampler${XD}${MS}(myTexture, mySampler),
          ivec2(gl_GlobalInvocationID.xy), level));
      }
      `;

    return this.device.createComputePipeline({
      layout: undefined,
      computeStage: {
        entryPoint: 'main',
        module: this.makeShaderModule('compute', { glsl }),
      },
    });
  }

  checkContents(texture, state, subresourceRange) {
    assert(this.params.dimension === '2d');

    const sampler = this.device.createSampler();

    for (const { level, slices } of subresourceRange.mipLevels()) {
      const width = this.textureWidth >> level;
      const height = this.textureHeight >> level;

      let readbackTypedArray = Float32Array;
      let prefix = '';
      let expectedShaderValue = initializedStateAsFloat(state);
      if (this.params.format.indexOf('sint') !== -1) {
        prefix = 'i';
        expectedShaderValue = initializedStateAsSint(state);
        readbackTypedArray = Int32Array;
      } else if (this.params.format.indexOf('uint') !== -1) {
        prefix = 'u';
        expectedShaderValue = initializedStateAsUint(state);
        readbackTypedArray = Uint32Array;
      }

      const computePipeline = this.getSamplingReadbackPipeline(
        prefix,
        this.params.sampleCount,
        this.params.dimension
      );

      for (const slice of slices) {
        const [ubo, uboMapping] = this.device.createBufferMapped({
          size: 4,
          usage: GPUBufferUsage.UNIFORM | GPUBufferUsage.COPY_DST,
        });

        new Int32Array(uboMapping, 0, 1)[0] = level;
        ubo.unmap();

        const byteLength =
          width *
          height *
          Uint32Array.BYTES_PER_ELEMENT *
          getTexelDataRepresentation(this.params.format).componentOrder.length;
        const resultBuffer = this.device.createBuffer({
          size: byteLength,
          usage: GPUBufferUsage.STORAGE | GPUBufferUsage.COPY_SRC,
        });

        const bindGroup = this.device.createBindGroup({
          layout: computePipeline.getBindGroupLayout(0),
          entries: [
            {
              binding: 0,
              resource: { buffer: ubo },
            },

            {
              binding: 1,
              resource: texture.createView({
                baseMipLevel: 0,
                mipLevelCount: this.params.mipLevelCount,
                baseArrayLayer: slice,
                arrayLayerCount: 1,
              }),
            },

            { binding: 2, resource: sampler },
            {
              binding: 3,
              resource: {
                buffer: resultBuffer,
              },
            },
          ],
        });

        const commandEncoder = this.device.createCommandEncoder();
        const pass = commandEncoder.beginComputePass();
        pass.setPipeline(computePipeline);
        pass.setBindGroup(0, bindGroup);
        pass.dispatch(width, height);
        pass.endPass();
        this.queue.submit([commandEncoder.finish()]);
        ubo.destroy();

        const mappedResultBuffer = this.createCopyForMapRead(resultBuffer, 0, byteLength);
        resultBuffer.destroy();

        this.eventualAsyncExpectation(async niceStack => {
          const actual = await mappedResultBuffer.mapReadAsync();
          const expected = new readbackTypedArray(new ArrayBuffer(actual.byteLength));
          expected.fill(expectedShaderValue);

          // TODO: Have a better way to determine approximately equal, maybe in ULPs.
          let tolerance;
          if (this.params.format === 'rgb10a2unorm') {
            tolerance = i => {
              // The alpha component is only two bits. Use a generous tolerance.
              return i % 4 === 3 ? 0.18 : 0.01;
            };
          } else {
            tolerance = 0.01;
          }

          const check = this.checkBuffer(new readbackTypedArray(actual), expected, tolerance);
          if (check !== undefined) {
            niceStack.message = check;
            this.rec.expectationFailed(niceStack);
          }
          mappedResultBuffer.destroy();
        });
      }
    }
  }
}

export const g = makeTestGroup(SampledTextureClearTest);

g.test('uninitialized_texture_is_zero')
  .params(TextureZeroInitTest.generateParams([ReadMethod.Sample]))
  .fn(t => t.run());
