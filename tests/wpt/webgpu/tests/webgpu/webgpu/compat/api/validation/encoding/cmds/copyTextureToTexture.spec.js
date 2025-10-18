/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Tests limitations of copyTextureToTextures in compat mode.
`;import { makeTestGroup } from '../../../../../../common/framework/test_group.js';
import {
  getBlockInfoForColorTextureFormat,
  getBlockInfoForTextureFormat,
  kCompressedTextureFormats,
  kPossibleMultisampledTextureFormats } from
'../../../../../format_info.js';
import { CompatibilityTest } from '../../../../compatibility_test.js';

export const g = makeTestGroup(CompatibilityTest);

g.test('compressed').
desc(`Tests that you can not call copyTextureToTexture with compressed textures in compat mode.`).
params((u) => u.combine('format', kCompressedTextureFormats)).
fn((t) => {
  const { format } = t.params;
  t.skipIfTextureFormatNotSupported(format);

  const { blockWidth, blockHeight } = getBlockInfoForColorTextureFormat(format);

  const srcTexture = t.createTextureTracked({
    size: [blockWidth, blockHeight, 1],
    format,
    usage: GPUTextureUsage.COPY_SRC
  });

  const dstTexture = t.createTextureTracked({
    size: [blockWidth, blockHeight, 1],
    format,
    usage: GPUTextureUsage.COPY_DST
  });

  const encoder = t.device.createCommandEncoder();
  encoder.copyTextureToTexture({ texture: srcTexture }, { texture: dstTexture }, [
  blockWidth,
  blockHeight,
  1]
  );
  t.expectGPUErrorInCompatibilityMode('validation', () => {
    encoder.finish();
  });
});

g.test('multisample').
desc(`Test that you can not call copyTextureToTexture with multisample textures in compat mode.`).
params((u) => u.combine('format', kPossibleMultisampledTextureFormats)).
fn((t) => {
  const { format } = t.params;
  const { blockWidth, blockHeight } = getBlockInfoForTextureFormat(format);

  t.skipIfTextureFormatNotSupported(format);
  t.skipIfTextureFormatNotMultisampled(format);

  const srcTexture = t.createTextureTracked({
    size: [blockWidth, blockHeight, 1],
    format,
    sampleCount: 4,
    usage: GPUTextureUsage.COPY_SRC | GPUTextureUsage.RENDER_ATTACHMENT
  });

  const dstTexture = t.createTextureTracked({
    size: [blockWidth, blockHeight, 1],
    format,
    sampleCount: 4,
    usage: GPUTextureUsage.COPY_DST | GPUTextureUsage.RENDER_ATTACHMENT
  });

  const encoder = t.device.createCommandEncoder();
  encoder.copyTextureToTexture({ texture: srcTexture }, { texture: dstTexture }, [
  blockWidth,
  blockHeight,
  1]
  );
  t.expectGPUErrorInCompatibilityMode('validation', () => {
    encoder.finish();
  });
});