/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Validation tests for the 'texture-component-swizzle' feature.

Test that:
* when the feature is off, swizzling is not allowed, even the identity swizzle.
* swizzling is not allowed on textures with usage STORAGE_BINDING nor RENDER_ATTACHMENT
  except the identity swizzle.
`;import { makeTestGroup } from '../../../../../common/framework/test_group.js';
import { UniqueFeaturesOrLimitsGPUTest } from '../../../../gpu_test.js';

import { isIdentitySwizzle, kSwizzleTests } from './texture_component_swizzle_utils.js';

export const g = makeTestGroup(UniqueFeaturesOrLimitsGPUTest);

g.test('invalid_swizzle').
desc(
  `
  Test that setting an invalid swizzle value on a texture view throws an exception.
  `
).
params((u) =>
u.beginSubcases().combine('invalidSwizzle', [
'rgbA', // swizzles are case-sensitive
'RGBA', // swizzles are case-sensitive
'rgb', // must have 4 components
'rgba01',
'ɲgba', // r with 0x200 added to each code point to make sure values are not truncated.
'ɲɧɢɡ', // rgba with 0x200 added to each code point to make sure values are not truncated.
'𝐫𝐠𝐛𝐚', // various unicode values that normalize to rgba
'𝑟𝑔𝑏𝑎',
'𝗋𝗀𝖻𝖺',
'𝓇ℊ𝒷𝒶',
'ⓡⓖⓑⓐ',
'ｒｇｂａ',
'ʳᵍᵇᵃ',
'000',
'00000',
'111',
'11111',
0,
1,
1111, // passes because toString is '1111'
1234,
1111.1,
0x72676261, // big endian rgba
0x61626772, // little endian rgba
0x30303030, // 0000
0x31313131, // 1111
true,
false,
null]
)
).
beforeAllSubcases((t) => {
  t.selectDeviceOrSkipTestCase('texture-component-swizzle');
}).
fn((t) => {
  const { invalidSwizzle } = t.params;
  const texture = t.createTextureTracked({
    format: 'rgba8unorm',
    size: [1],
    usage: GPUTextureUsage.COPY_DST | GPUTextureUsage.TEXTURE_BINDING
  });

  const failure = typeof invalidSwizzle !== 'number' || invalidSwizzle !== 1111;
  t.shouldThrow(failure ? 'TypeError' : false, () => {
    texture.createView({ swizzle: invalidSwizzle });
  });
});

g.test('only_identity_swizzle').
desc(
  `
  Test that if texture-component-swizzle is not enabled, having a non-default swizzle property generates a validation error.
  `
).
params((u) => u.beginSubcases().combine('swizzle', kSwizzleTests)).
fn((t) => {
  const { swizzle } = t.params;
  const texture = t.createTextureTracked({
    format: 'rgba8unorm',
    size: [1],
    usage: GPUTextureUsage.COPY_DST | GPUTextureUsage.TEXTURE_BINDING
  });
  const shouldError = !isIdentitySwizzle(swizzle);
  t.expectValidationError(() => {
    texture.createView({ swizzle });
  }, shouldError);
});

g.test('no_render_no_resolve_no_storage').
desc(
  `
  Test that setting a non-identity swizzle gets an error if used as a render attachment,
  a resolve target, or a storage binding.
  `
).
params((u) =>
u.
combine('useCase', [
'texture-binding',
'color-attachment',
'depth-attachment',
'stencil-attachment',
'resolve-target',
'storage-binding']
).
beginSubcases().
combine('swizzle', kSwizzleTests)
).
beforeAllSubcases((t) => {
  t.selectDeviceOrSkipTestCase('texture-component-swizzle');
}).
fn((t) => {
  const { swizzle, useCase } = t.params;
  const texture = t.createTextureTracked({
    format:
    useCase === 'depth-attachment' ?
    'depth16unorm' :
    useCase === 'stencil-attachment' ?
    'stencil8' :
    'rgba8unorm',
    size: [1],
    usage:
    GPUTextureUsage.COPY_SRC |
    GPUTextureUsage.COPY_DST |
    GPUTextureUsage.TEXTURE_BINDING |
    GPUTextureUsage.RENDER_ATTACHMENT | (
    useCase === 'storage-binding' ? GPUTextureUsage.STORAGE_BINDING : 0)
  });
  const view = texture.createView({ swizzle });
  const shouldError = useCase !== 'texture-binding' && !isIdentitySwizzle(swizzle);
  switch (useCase) {
    case 'texture-binding':{
        const bindGroupLayout = t.device.createBindGroupLayout({
          entries: [
          {
            binding: 0,
            visibility: GPUShaderStage.FRAGMENT,
            texture: {}
          }]

        });
        t.expectValidationError(() => {
          t.device.createBindGroup({
            layout: bindGroupLayout,
            entries: [{ binding: 0, resource: view }]
          });
        }, shouldError);
        break;
      }
    case 'color-attachment':{
        t.expectValidationError(() => {
          const encoder = t.device.createCommandEncoder();
          const pass = encoder.beginRenderPass({
            colorAttachments: [
            {
              view,
              loadOp: 'clear',
              storeOp: 'store'
            }]

          });
          pass.end();
          encoder.finish();
        }, shouldError);
        break;
      }
    case 'depth-attachment':{
        t.expectValidationError(() => {
          const encoder = t.device.createCommandEncoder();
          const pass = encoder.beginRenderPass({
            colorAttachments: [],
            depthStencilAttachment: {
              view,
              depthClearValue: 1,
              depthLoadOp: 'clear',
              depthStoreOp: 'store'
            }
          });
          pass.end();
          encoder.finish();
        }, shouldError);
        break;
      }
    case 'stencil-attachment':{
        t.expectValidationError(() => {
          const encoder = t.device.createCommandEncoder();
          const pass = encoder.beginRenderPass({
            colorAttachments: [],
            depthStencilAttachment: {
              view,
              stencilClearValue: 0,
              stencilLoadOp: 'clear',
              stencilStoreOp: 'store'
            }
          });
          pass.end();
          encoder.finish();
        }, shouldError);
        break;
      }
    case 'resolve-target':{
        t.expectValidationError(() => {
          const encoder = t.device.createCommandEncoder();
          const pass = encoder.beginRenderPass({
            colorAttachments: [
            {
              view: t.createTextureTracked({
                format: 'rgba8unorm',
                size: [1],
                usage: GPUTextureUsage.RENDER_ATTACHMENT,
                sampleCount: 4
              }),
              resolveTarget: view,
              loadOp: 'clear',
              storeOp: 'store'
            }]

          });
          pass.end();
          encoder.finish();
        }, shouldError);
        break;
      }
    case 'storage-binding':{
        const bindGroupLayout = t.device.createBindGroupLayout({
          entries: [
          {
            binding: 0,
            visibility: GPUShaderStage.COMPUTE,
            storageTexture: {
              access: 'read-only',
              format: 'rgba8unorm'
            }
          }]

        });
        t.expectValidationError(() => {
          t.device.createBindGroup({
            layout: bindGroupLayout,
            entries: [{ binding: 0, resource: view }]
          });
        }, shouldError);
        break;
      }
  }
});

g.test('compatibility_mode').
desc(
  `
  Test that in compatibility mode, swizzles must be equivalent.
  `
).
beforeAllSubcases((t) => {
  t.selectDeviceOrSkipTestCase('texture-component-swizzle');
}).
params((u) =>
u.
beginSubcases().
combine('swizzle', kSwizzleTests).
combine('otherSwizzle', kSwizzleTests).
combine('pipelineType', ['render', 'compute'])
).
fn((t) => {
  const { swizzle, otherSwizzle, pipelineType } = t.params;

  const module = t.device.createShaderModule({
    code: `
        @group(0) @binding(0) var tex0: texture_2d<f32>;
        @group(1) @binding(0) var tex1: texture_2d<f32>;

        @compute @workgroup_size(1) fn cs() {
          _ = tex0;
          _ = tex1;
        }

        @vertex fn vs() -> @builtin(position) vec4f {
          return vec4f(0);
        }

        @fragment fn fs() -> @location(0) vec4f {
          _ = tex0;
          _ = tex1;
          return vec4f(0);
        }
      `
  });

  const pipeline =
  pipelineType === 'compute' ?
  t.device.createComputePipeline({
    layout: 'auto',
    compute: { module }
  }) :
  t.device.createRenderPipeline({
    layout: 'auto',
    vertex: { module },
    fragment: { module, targets: [{ format: 'rgba8unorm' }] }
  });

  const texture = t.createTextureTracked({
    size: [1],
    format: 'rgba8unorm',
    usage: GPUTextureUsage.TEXTURE_BINDING
  });

  const bindGroup0 = t.device.createBindGroup({
    layout: pipeline.getBindGroupLayout(0),
    entries: [
    {
      binding: 0,
      resource: texture.createView({ swizzle })
    }]

  });

  const bindGroup1 = t.device.createBindGroup({
    layout: pipeline.getBindGroupLayout(0),
    entries: [
    {
      binding: 0,
      resource: texture.createView({ swizzle: otherSwizzle })
    }]

  });

  const encoder = t.device.createCommandEncoder();
  switch (pipelineType) {
    case 'compute':{
        const pass = encoder.beginComputePass();
        pass.setPipeline(pipeline);
        pass.setBindGroup(0, bindGroup0);
        pass.setBindGroup(1, bindGroup1);
        pass.dispatchWorkgroups(1);
        pass.end();
        break;
      }
    case 'render':{
        const view = t.createTextureTracked({
          size: [1],
          format: 'rgba8unorm',
          usage: GPUTextureUsage.RENDER_ATTACHMENT
        });
        const pass = encoder.beginRenderPass({
          colorAttachments: [{ view, loadOp: 'clear', storeOp: 'store' }]
        });
        pass.setPipeline(pipeline);
        pass.setBindGroup(0, bindGroup0);
        pass.setBindGroup(1, bindGroup1);
        pass.draw(3);
        pass.end();
      }
  }

  const shouldError = t.isCompatibility && swizzle !== otherSwizzle;

  t.expectValidationError(() => {
    encoder.finish();
  }, shouldError);
});