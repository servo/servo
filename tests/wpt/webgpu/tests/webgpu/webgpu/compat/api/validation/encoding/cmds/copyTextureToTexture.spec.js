/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Tests limitations of copyTextureToTextures in compat mode.
`;import { makeTestGroup } from '../../../../../../common/framework/test_group.js';
import { kCompressedTextureFormats, kTextureFormatInfo } from '../../../../../format_info.js';
import { CompatibilityTest } from '../../../../compatibility_test.js';

export const g = makeTestGroup(CompatibilityTest);

g.test('compressed').
desc(
  `Tests that you can not call copyTextureToTextures with compressed textures in compat mode.`
).
params((u) => u.combine('format', kCompressedTextureFormats)).
beforeAllSubcases((t) => {
  const { format } = t.params;
  t.selectDeviceOrSkipTestCase([kTextureFormatInfo[format].feature]);
}).
fn((t) => {
  const { format } = t.params;

  const { blockWidth, blockHeight } = kTextureFormatInfo[format];

  const srcTexture = t.device.createTexture({
    size: [blockWidth, blockHeight, 1],
    format,
    usage: GPUTextureUsage.COPY_SRC
  });
  t.trackForCleanup(srcTexture);

  const dstTexture = t.device.createTexture({
    size: [blockWidth, blockHeight, 1],
    format,
    usage: GPUTextureUsage.COPY_DST
  });
  t.trackForCleanup(dstTexture);

  const encoder = t.device.createCommandEncoder();
  encoder.copyTextureToTexture({ texture: srcTexture }, { texture: dstTexture }, [
  blockWidth,
  blockHeight,
  1]
  );
  t.expectGPUError('validation', () => {
    encoder.finish();
  });
});