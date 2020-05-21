/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/

export const description = `
createTexture validation tests.
`;
import { poptions } from '../../../common/framework/params_builder.js';
import { makeTestGroup } from '../../../common/framework/test_group.js';
import { kTextureFormatInfo, kTextureFormats } from '../../capability_info.js';
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
        depth: arrayLayerCount
      },
      mipLevelCount,
      sampleCount,
      dimension: '2d',
      format,
      usage: GPUTextureUsage.OUTPUT_ATTACHMENT | GPUTextureUsage.SAMPLED
    };
  }

}

export const g = makeTestGroup(F);
g.test('validation_of_sampleCount').params([// TODO: Consider making a list of "valid"+"invalid" texture descriptors in capability_info.
{
  sampleCount: 0,
  _success: false
}, // sampleCount of 0 is not allowed
{
  sampleCount: 1,
  _success: true
}, // sampleCount of 1 is allowed
{
  sampleCount: 2,
  _success: false
}, // sampleCount of 2 is not allowed
{
  sampleCount: 3,
  _success: false
}, // sampleCount of 3 is not allowed
{
  sampleCount: 4,
  _success: true
}, // sampleCount of 4 is allowed
{
  sampleCount: 8,
  _success: false
}, // sampleCount of 8 is not allowed
{
  sampleCount: 16,
  _success: false
}, // sampleCount of 16 is not allowed
{
  sampleCount: 4,
  mipLevelCount: 2,
  _success: false
}, // multisampled multi-level not allowed
{
  sampleCount: 4,
  arrayLayerCount: 2,
  _success: false
} // multisampled multi-layer is not allowed
]).fn(async t => {
  const {
    sampleCount,
    mipLevelCount,
    arrayLayerCount,
    _success
  } = t.params;
  const descriptor = t.getDescriptor({
    sampleCount,
    mipLevelCount,
    arrayLayerCount
  });
  t.expectValidationError(() => {
    t.device.createTexture(descriptor);
  }, !_success);
});
g.test('validation_of_mipLevelCount').params([{
  width: 32,
  height: 32,
  mipLevelCount: 1,
  _success: true
}, // mipLevelCount of 1 is allowed
{
  width: 32,
  height: 32,
  mipLevelCount: 0,
  _success: false
}, // mipLevelCount of 0 is not allowed
{
  width: 32,
  height: 32,
  mipLevelCount: 6,
  _success: true
}, // full mip chains are allowed (Mip level sizes: 32, 16, 8, 4, 2, 1)
{
  width: 31,
  height: 32,
  mipLevelCount: 6,
  _success: true
}, // full mip chains are allowed (Mip level sizes: 31x32, 15x16, 7x8, 3x4, 1x2, 1x1)
{
  width: 32,
  height: 31,
  mipLevelCount: 6,
  _success: true
}, // full mip chains are allowed (Mip level sizes: 32x31, 16x15, 8x7, 4x3, 2x1, 1x1)
{
  width: 31,
  height: 32,
  mipLevelCount: 7,
  _success: false
}, // too big mip chains on width are disallowed (Mip level sizes: 31x32, 15x16, 7x8, 3x4, 1x2, 1x1, ?x?)
{
  width: 32,
  height: 31,
  mipLevelCount: 7,
  _success: false
}, // too big mip chains on height are disallowed (Mip level sizes: 32x31, 16x15, 8x7, 4x3, 2x1, 1x1, ?x?)
{
  width: 32,
  height: 32,
  mipLevelCount: 100,
  _success: false
}, // undefined shift check if miplevel is bigger than the integer bit width
{
  width: 32,
  height: 8,
  mipLevelCount: 6,
  _success: true
} // non square mip map halves the resolution until a 1x1 dimension. (Mip maps: 32 * 8, 16 * 4, 8 * 2, 4 * 1, 2 * 1, 1 * 1)
]).fn(async t => {
  const {
    width,
    height,
    mipLevelCount,
    _success
  } = t.params;
  const descriptor = t.getDescriptor({
    width,
    height,
    mipLevelCount
  });
  t.expectValidationError(() => {
    t.device.createTexture(descriptor);
  }, !_success);
});
g.test('it_is_valid_to_destroy_a_texture').fn(t => {
  const descriptor = t.getDescriptor();
  const texture = t.device.createTexture(descriptor);
  texture.destroy();
});
g.test('it_is_valid_to_destroy_a_destroyed_texture').fn(t => {
  const descriptor = t.getDescriptor();
  const texture = t.device.createTexture(descriptor);
  texture.destroy();
  texture.destroy();
});
g.test('it_is_invalid_to_submit_a_destroyed_texture_before_and_after_encode').params([{
  destroyBeforeEncode: false,
  destroyAfterEncode: false,
  _success: true
}, {
  destroyBeforeEncode: true,
  destroyAfterEncode: false,
  _success: false
}, {
  destroyBeforeEncode: false,
  destroyAfterEncode: true,
  _success: false
}]).fn(async t => {
  const {
    destroyBeforeEncode,
    destroyAfterEncode,
    _success
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

  t.expectValidationError(() => {
    t.queue.submit([commandBuffer]);
  }, !_success);
});
g.test('it_is_invalid_to_have_an_output_attachment_texture_with_non_renderable_format').params(poptions('format', kTextureFormats)).fn(async t => {
  const format = t.params.format;
  const info = kTextureFormatInfo[format];
  const descriptor = t.getDescriptor({
    width: 1,
    height: 1,
    format
  });
  t.expectValidationError(() => {
    t.device.createTexture(descriptor);
  }, !info.renderable);
}); // TODO: Add tests for compressed texture formats
//# sourceMappingURL=createTexture.spec.js.map