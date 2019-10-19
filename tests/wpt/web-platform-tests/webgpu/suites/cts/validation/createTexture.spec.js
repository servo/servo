/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/

export const description = `
createTexture validation tests.
`;
import { TestGroup } from '../../../framework/index.js';
import { ValidationTest } from './validation_test.js';

class F extends ValidationTest {
  getDescriptor(options = {}) {
    const {
      width = 32,
      height = 32,
      arrayLayerCount = 1,
      mipLevelCount = 1,
      sampleCount = 1,
      format = 'rgba8unorm'
    } = options;
    return {
      size: {
        width,
        height,
        depth: 1
      },
      arrayLayerCount,
      mipLevelCount,
      sampleCount,
      dimension: '2d',
      format,
      usage: GPUTextureUsage.OUTPUT_ATTACHMENT | GPUTextureUsage.SAMPLED
    };
  }

}

export const g = new TestGroup(F);
g.test('validation of sampleCount', async t => {
  const {
    sampleCount,
    mipLevelCount,
    arrayLayerCount,
    success
  } = t.params;
  const descriptor = t.getDescriptor({
    sampleCount,
    mipLevelCount,
    arrayLayerCount
  });
  await t.expectValidationError(() => {
    t.device.createTexture(descriptor);
  }, !success);
}).params([{
  sampleCount: 0,
  success: false
}, // sampleCount of 0 is not allowed
{
  sampleCount: 1,
  success: true
}, // sampleCount of 1 is allowed
{
  sampleCount: 2,
  success: false
}, // sampleCount of 2 is not allowed
{
  sampleCount: 3,
  success: false
}, // sampleCount of 3 is not allowed
{
  sampleCount: 4,
  success: true
}, // sampleCount of 4 is allowed
{
  sampleCount: 8,
  success: false
}, // sampleCount of 8 is not allowed
{
  sampleCount: 16,
  success: false
}, // sampleCount of 16 is not allowed
{
  sampleCount: 4,
  mipLevelCount: 2,
  success: false
}, // it is an error to create a multisampled texture with mipLevelCount > 1
{
  sampleCount: 4,
  arrayLayerCount: 2,
  success: true
} // multisampled 2D array texture is supported
]);
g.test('validation of mipLevelCount', async t => {
  const {
    width,
    height,
    mipLevelCount,
    success
  } = t.params;
  const descriptor = t.getDescriptor({
    width,
    height,
    mipLevelCount
  });
  await t.expectValidationError(() => {
    t.device.createTexture(descriptor);
  }, !success);
}).params([{
  width: 32,
  height: 32,
  mipLevelCount: 1,
  success: true
}, // mipLevelCount of 1 is allowed
{
  width: 32,
  height: 32,
  mipLevelCount: 0,
  success: false
}, // mipLevelCount of 0 is not allowed
{
  width: 32,
  height: 32,
  mipLevelCount: 6,
  success: true
}, // full mip chains are allowed (Mip level sizes: 32, 16, 8, 4, 2, 1)
{
  width: 31,
  height: 32,
  mipLevelCount: 7,
  success: false
}, // too big mip chains on width are disallowed (Mip level width: 31, 15, 7, 3, 1, 1)
{
  width: 32,
  height: 31,
  mipLevelCount: 7,
  success: false
}, // too big mip chains on height are disallowed (Mip level width: 31, 15, 7, 3, 1, 1)
{
  width: 32,
  height: 32,
  mipLevelCount: 100,
  success: false
}, // undefined shift check if miplevel is bigger than the integer bit width
{
  width: 32,
  height: 8,
  mipLevelCount: 6,
  success: true
} // non square mip map halves the resolution until a 1x1 dimension. (Mip maps: 32 * 8, 16 * 4, 8 * 2, 4 * 1, 2 * 1, 1 * 1)
]);
g.test('it is valid to destroy a texture', t => {
  const descriptor = t.getDescriptor();
  const texture = t.device.createTexture(descriptor);
  texture.destroy();
});
g.test('it is valid to destroy a destroyed texture', t => {
  const descriptor = t.getDescriptor();
  const texture = t.device.createTexture(descriptor);
  texture.destroy();
  texture.destroy();
});
g.test('it is invalid to submit a destroyed texture before and after encode', async t => {
  const {
    destroyBeforeEncode,
    destroyAfterEncode,
    success
  } = t.params;
  const descriptor = t.getDescriptor();
  const texture = t.device.createTexture(descriptor);
  const textureView = texture.createView();

  if (destroyBeforeEncode) {
    texture.destroy();
  }

  const commandEncoder = t.device.createCommandEncoder();
  const renderPass = commandEncoder.beginRenderPass({
    colorAttachments: [{
      attachment: textureView,
      loadValue: {
        r: 1.0,
        g: 0.0,
        b: 0.0,
        a: 1.0
      }
    }]
  });
  renderPass.endPass();
  const commandBuffer = commandEncoder.finish();

  if (destroyAfterEncode) {
    texture.destroy();
  }

  await t.expectValidationError(() => {
    t.queue.submit([commandBuffer]);
  }, !success);
}).params([{
  destroyBeforeEncode: false,
  destroyAfterEncode: false,
  success: true
}, {
  destroyBeforeEncode: true,
  destroyAfterEncode: false,
  success: false
}, {
  destroyBeforeEncode: false,
  destroyAfterEncode: true,
  success: false
}]);
g.test('it is invalid to have an output attachment texture with non renderable format', async t => {
  const {
    format,
    success
  } = t.params;
  const descriptor = t.getDescriptor({
    width: 1,
    height: 1,
    format
  });
  await t.expectValidationError(() => {
    t.device.createTexture(descriptor);
  }, !success);
}).params([// 8-bit formats
{
  format: 'r8unorm',
  success: true
}, {
  format: 'r8snorm',
  success: false
}, {
  format: 'r8uint',
  success: true
}, {
  format: 'r8sint',
  success: true
}, // 16-bit formats
{
  format: 'r16uint',
  success: true
}, {
  format: 'r16sint',
  success: true
}, {
  format: 'r16float',
  success: true
}, {
  format: 'rg8unorm',
  success: true
}, {
  format: 'rg8snorm',
  success: false
}, {
  format: 'rg8uint',
  success: true
}, {
  format: 'rg8sint',
  success: true
}, // 32-bit formats
{
  format: 'r32uint',
  success: true
}, {
  format: 'r32sint',
  success: true
}, {
  format: 'r32float',
  success: true
}, {
  format: 'rg16uint',
  success: true
}, {
  format: 'rg16sint',
  success: true
}, {
  format: 'rg16float',
  success: true
}, {
  format: 'rgba8unorm',
  success: true
}, {
  format: 'rgba8unorm-srgb',
  success: true
}, {
  format: 'rgba8snorm',
  success: false
}, {
  format: 'rgba8uint',
  success: true
}, {
  format: 'rgba8sint',
  success: true
}, {
  format: 'bgra8unorm',
  success: true
}, {
  format: 'bgra8unorm-srgb',
  success: true
}, // Packed 32-bit formats
{
  format: 'rgb10a2unorm',
  success: true
}, {
  format: 'rg11b10float',
  success: false
}, // 64-bit formats
{
  format: 'rg32uint',
  success: true
}, {
  format: 'rg32sint',
  success: true
}, {
  format: 'rg32float',
  success: true
}, {
  format: 'rgba16uint',
  success: true
}, {
  format: 'rgba16sint',
  success: true
}, {
  format: 'rgba16float',
  success: true
}, // 128-bit formats
{
  format: 'rgba32uint',
  success: true
}, {
  format: 'rgba32sint',
  success: true
}, {
  format: 'rgba32float',
  success: true
}]); // TODO: Add tests for compressed texture formats
//# sourceMappingURL=createTexture.spec.js.map