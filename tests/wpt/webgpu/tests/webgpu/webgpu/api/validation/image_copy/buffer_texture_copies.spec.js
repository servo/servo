/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
copyTextureToBuffer and copyBufferToTexture validation tests not covered by
the general image_copy tests, or by destroyed,*.
`;import { makeTestGroup } from '../../../../common/framework/test_group.js';
import { assert, unreachable } from '../../../../common/util/util.js';
import { kBufferUsages, kTextureUsages } from '../../../capability_info.js';
import { GPUConst } from '../../../constants.js';
import {
  kDepthStencilFormats,
  depthStencilBufferTextureCopySupported,
  depthStencilFormatAspectSize } from
'../../../format_info.js';
import { align } from '../../../util/math.js';
import { kBufferCopyAlignment, kBytesPerRowAlignment } from '../../../util/texture/layout.js';
import { ValidationTest } from '../validation_test.js';

class ImageCopyTest extends ValidationTest {
  testCopyBufferToTexture(
  source,
  destination,
  copySize,
  isSuccess)
  {
    const { encoder, validateFinishAndSubmit } = this.createEncoder('non-pass');
    encoder.copyBufferToTexture(source, destination, copySize);
    validateFinishAndSubmit(isSuccess, true);
  }

  testCopyTextureToBuffer(
  source,
  destination,
  copySize,
  isSuccess)
  {
    const { encoder, validateFinishAndSubmit } = this.createEncoder('non-pass');
    encoder.copyTextureToBuffer(source, destination, copySize);
    validateFinishAndSubmit(isSuccess, true);
  }

  testWriteTexture(
  destination,
  uploadData,
  dataLayout,
  copySize,
  isSuccess)
  {
    this.expectGPUError(
      'validation',
      () => this.queue.writeTexture(destination, uploadData, dataLayout, copySize),
      !isSuccess
    );
  }
}

export const g = makeTestGroup(ImageCopyTest);

g.test('depth_stencil_format,copy_usage_and_aspect').
desc(
  `
  Validate the combination of usage and aspect of each depth stencil format in copyBufferToTexture,
  copyTextureToBuffer and writeTexture. See https://gpuweb.github.io/gpuweb/#depth-formats for more
  details.
  `
).
params((u) =>
u //
.combine('format', kDepthStencilFormats).
beginSubcases().
combine('aspect', ['all', 'depth-only', 'stencil-only'])
).
beforeAllSubcases((t) => {
  const { format } = t.params;
  t.selectDeviceForTextureFormatOrSkipTestCase(format);
}).
fn((t) => {
  const { format, aspect } = t.params;

  const textureSize = { width: 1, height: 1, depthOrArrayLayers: 1 };
  const texture = t.device.createTexture({
    size: textureSize,
    format,
    usage: GPUTextureUsage.COPY_SRC | GPUTextureUsage.COPY_DST
  });

  const uploadBufferSize = 32;
  const buffer = t.device.createBuffer({
    size: uploadBufferSize,
    usage: GPUBufferUsage.COPY_SRC | GPUBufferUsage.COPY_DST
  });

  {
    const success = depthStencilBufferTextureCopySupported('CopyB2T', format, aspect);
    t.testCopyBufferToTexture({ buffer }, { texture, aspect }, textureSize, success);
  }

  {
    const success = depthStencilBufferTextureCopySupported('CopyT2B', format, aspect);
    t.testCopyTextureToBuffer({ texture, aspect }, { buffer }, textureSize, success);
  }

  {
    const success = depthStencilBufferTextureCopySupported('WriteTexture', format, aspect);
    const uploadData = new Uint8Array(uploadBufferSize);
    t.testWriteTexture({ texture, aspect }, uploadData, {}, textureSize, success);
  }
});

g.test('depth_stencil_format,copy_buffer_size').
desc(
  `
  Validate the minimum buffer size for each depth stencil format in copyBufferToTexture,
  copyTextureToBuffer and writeTexture.

  Given a depth stencil format, a copy aspect ('depth-only' or 'stencil-only'), the copy method
  (buffer-to-texture or texture-to-buffer) and the copy size, validate
  - if the copy can be successfully executed with the minimum required buffer size.
  - if the copy fails with a validation error when the buffer size is less than the minimum
  required buffer size.
  `
).
params((u) =>
u.
combine('format', kDepthStencilFormats).
combine('aspect', ['depth-only', 'stencil-only']).
combine('copyType', ['CopyB2T', 'CopyT2B', 'WriteTexture']).
filter((param) =>
depthStencilBufferTextureCopySupported(param.copyType, param.format, param.aspect)
).
beginSubcases().
combine('copySize', [
{ width: 8, height: 1, depthOrArrayLayers: 1 },
{ width: 4, height: 4, depthOrArrayLayers: 1 },
{ width: 4, height: 4, depthOrArrayLayers: 3 }]
)
).
beforeAllSubcases((t) => {
  const { format } = t.params;
  t.selectDeviceForTextureFormatOrSkipTestCase(format);
}).
fn((t) => {
  const { format, aspect, copyType, copySize } = t.params;

  const texture = t.device.createTexture({
    size: copySize,
    format,
    usage: GPUTextureUsage.COPY_SRC | GPUTextureUsage.COPY_DST
  });

  const texelAspectSize = depthStencilFormatAspectSize(format, aspect);
  assert(texelAspectSize > 0);

  const bytesPerRowAlignment = copyType === 'WriteTexture' ? 1 : kBytesPerRowAlignment;
  const bytesPerRow = align(texelAspectSize * copySize.width, bytesPerRowAlignment);
  const rowsPerImage = copySize.height;
  const minimumBufferSize =
  bytesPerRow * (rowsPerImage * copySize.depthOrArrayLayers - 1) +
  align(texelAspectSize * copySize.width, kBufferCopyAlignment);
  assert(minimumBufferSize > kBufferCopyAlignment);

  const bigEnoughBuffer = t.device.createBuffer({
    size: minimumBufferSize,
    usage: GPUBufferUsage.COPY_SRC | GPUBufferUsage.COPY_DST
  });
  const smallerBuffer = t.device.createBuffer({
    size: minimumBufferSize - kBufferCopyAlignment,
    usage: GPUBufferUsage.COPY_SRC | GPUBufferUsage.COPY_DST
  });

  if (copyType === 'CopyB2T') {
    t.testCopyBufferToTexture(
      { buffer: bigEnoughBuffer, bytesPerRow, rowsPerImage },
      { texture, aspect },
      copySize,
      true
    );
    t.testCopyBufferToTexture(
      { buffer: smallerBuffer, bytesPerRow, rowsPerImage },
      { texture, aspect },
      copySize,
      false
    );
  } else if (copyType === 'CopyT2B') {
    t.testCopyTextureToBuffer(
      { texture, aspect },
      { buffer: bigEnoughBuffer, bytesPerRow, rowsPerImage },
      copySize,
      true
    );
    t.testCopyTextureToBuffer(
      { texture, aspect },
      { buffer: smallerBuffer, bytesPerRow, rowsPerImage },
      copySize,
      false
    );
  } else if (copyType === 'WriteTexture') {
    const enoughUploadData = new Uint8Array(minimumBufferSize);
    const smallerUploadData = new Uint8Array(minimumBufferSize - kBufferCopyAlignment);
    t.testWriteTexture(
      { texture, aspect },
      enoughUploadData,
      {
        bytesPerRow,
        rowsPerImage
      },
      copySize,
      true
    );

    t.testWriteTexture(
      { texture, aspect },
      smallerUploadData,
      {
        bytesPerRow,
        rowsPerImage
      },
      copySize,
      false
    );
  } else {
    unreachable();
  }
});

g.test('depth_stencil_format,copy_buffer_offset').
desc(
  `
    Validate for every depth stencil formats the buffer offset must be a multiple of 4 in
    copyBufferToTexture() and copyTextureToBuffer(), but the offset in writeTexture() doesn't always
    need to be a multiple of 4.
    `
).
params((u) =>
u.
combine('format', kDepthStencilFormats).
combine('aspect', ['depth-only', 'stencil-only']).
combine('copyType', ['CopyB2T', 'CopyT2B', 'WriteTexture']).
filter((param) =>
depthStencilBufferTextureCopySupported(param.copyType, param.format, param.aspect)
).
beginSubcases().
combine('offset', [1, 2, 4, 6, 8])
).
beforeAllSubcases((t) => {
  const { format } = t.params;
  t.selectDeviceForTextureFormatOrSkipTestCase(format);
}).
fn((t) => {
  const { format, aspect, copyType, offset } = t.params;

  const textureSize = { width: 4, height: 4, depthOrArrayLayers: 1 };

  const texture = t.device.createTexture({
    size: textureSize,
    format,
    usage: GPUTextureUsage.COPY_SRC | GPUTextureUsage.COPY_DST
  });

  const texelAspectSize = depthStencilFormatAspectSize(format, aspect);
  assert(texelAspectSize > 0);

  const bytesPerRowAlignment = copyType === 'WriteTexture' ? 1 : kBytesPerRowAlignment;
  const bytesPerRow = align(texelAspectSize * textureSize.width, bytesPerRowAlignment);
  const rowsPerImage = textureSize.height;
  const minimumBufferSize =
  bytesPerRow * (rowsPerImage * textureSize.depthOrArrayLayers - 1) +
  align(texelAspectSize * textureSize.width, kBufferCopyAlignment);
  assert(minimumBufferSize > kBufferCopyAlignment);

  const buffer = t.device.createBuffer({
    size: align(minimumBufferSize + offset, kBufferCopyAlignment),
    usage: GPUBufferUsage.COPY_SRC | GPUBufferUsage.COPY_DST
  });

  const isSuccess = copyType === 'WriteTexture' ? true : offset % 4 === 0;

  if (copyType === 'CopyB2T') {
    t.testCopyBufferToTexture(
      { buffer, offset, bytesPerRow, rowsPerImage },
      { texture, aspect },
      textureSize,
      isSuccess
    );
  } else if (copyType === 'CopyT2B') {
    t.testCopyTextureToBuffer(
      { texture, aspect },
      { buffer, offset, bytesPerRow, rowsPerImage },
      textureSize,
      isSuccess
    );
  } else if (copyType === 'WriteTexture') {
    const uploadData = new Uint8Array(minimumBufferSize + offset);
    t.testWriteTexture(
      { texture, aspect },
      uploadData,
      {
        offset,
        bytesPerRow,
        rowsPerImage
      },
      textureSize,
      isSuccess
    );
  } else {
    unreachable();
  }
});

g.test('sample_count').
desc(
  `
  Test that the texture sample count. Check that a validation error is generated if sample count is
  not 1.
  `
).
params((u) =>
u //
// writeTexture is handled by writeTexture.spec.ts.
.combine('copyType', ['CopyB2T', 'CopyT2B']).
beginSubcases().
combine('sampleCount', [1, 4])
).
fn((t) => {
  const { sampleCount, copyType } = t.params;

  let usage = GPUTextureUsage.COPY_SRC | GPUTextureUsage.COPY_DST;
  // WebGPU SPEC requires multisampled textures must have RENDER_ATTACHMENT usage.
  if (sampleCount > 1) {
    usage |= GPUTextureUsage.RENDER_ATTACHMENT;
  }
  const texture = t.device.createTexture({
    size: { width: 16, height: 16 },
    sampleCount,
    format: 'bgra8unorm',
    usage
  });

  const uploadBufferSize = 32;
  const buffer = t.device.createBuffer({
    size: uploadBufferSize,
    usage: GPUBufferUsage.COPY_SRC | GPUBufferUsage.COPY_DST
  });

  const textureSize = { width: 1, height: 1, depthOrArrayLayers: 1 };

  const isSuccess = sampleCount === 1;

  if (copyType === 'CopyB2T') {
    t.testCopyBufferToTexture({ buffer }, { texture }, textureSize, isSuccess);
  } else if (copyType === 'CopyT2B') {
    t.testCopyTextureToBuffer({ texture }, { buffer }, textureSize, isSuccess);
  }
});

const kRequiredTextureUsage = {
  CopyT2B: GPUConst.TextureUsage.COPY_SRC,
  CopyB2T: GPUConst.TextureUsage.COPY_DST
};
const kRequiredBufferUsage = {
  CopyB2T: GPUConst.BufferUsage.COPY_SRC,
  CopyT2B: GPUConst.BufferUsage.COPY_DST
};

g.test('texture_buffer_usages').
desc(
  `
  Tests calling copyTextureToBuffer or copyBufferToTexture with the texture and the buffer missed
  COPY_SRC, COPY_DST usage respectively.
    - texture and buffer {with, without} COPY_SRC and COPY_DST usage.
  `
).
params((u) =>
u //
.combine('copyType', ['CopyB2T', 'CopyT2B']).
beginSubcases().
combine('textureUsage', kTextureUsages).
expand('_textureUsageValid', (p) => [p.textureUsage === kRequiredTextureUsage[p.copyType]]).
combine('bufferUsage', kBufferUsages).
expand('_bufferUsageValid', (p) => [p.bufferUsage === kRequiredBufferUsage[p.copyType]]).
filter((p) => p._textureUsageValid || p._bufferUsageValid)
).
fn((t) => {
  const { copyType, textureUsage, _textureUsageValid, bufferUsage, _bufferUsageValid } = t.params;

  const texture = t.device.createTexture({
    size: { width: 16, height: 16 },
    format: 'rgba8unorm',
    usage: textureUsage
  });

  const uploadBufferSize = 32;
  const buffer = t.device.createBuffer({
    size: uploadBufferSize,
    usage: bufferUsage
  });

  const textureSize = { width: 1, height: 1, depthOrArrayLayers: 1 };

  const isSuccess = _textureUsageValid && _bufferUsageValid;
  if (copyType === 'CopyB2T') {
    t.testCopyBufferToTexture({ buffer }, { texture }, textureSize, isSuccess);
  } else if (copyType === 'CopyT2B') {
    t.testCopyTextureToBuffer({ texture }, { buffer }, textureSize, isSuccess);
  }
});

g.test('device_mismatch').
desc(
  `
    Tests copyBufferToTexture and copyTextureToBuffer cannot be called with a buffer or a texture
    created from another device.
  `
).
params((u) =>
u //
.combine('copyType', ['CopyB2T', 'CopyT2B']).
beginSubcases().
combineWithParams([
{ bufMismatched: false, texMismatched: false }, // control case
{ bufMismatched: true, texMismatched: false },
{ bufMismatched: false, texMismatched: true }]
)
).
beforeAllSubcases((t) => {
  t.selectMismatchedDeviceOrSkipTestCase(undefined);
}).
fn((t) => {
  const { copyType, bufMismatched, texMismatched } = t.params;

  const uploadBufferSize = 32;
  const buffer = (bufMismatched ? t.mismatchedDevice : t.device).createBuffer({
    size: uploadBufferSize,
    usage: GPUBufferUsage.COPY_SRC | GPUBufferUsage.COPY_DST
  });
  t.trackForCleanup(buffer);

  const textureSize = { width: 1, height: 1, depthOrArrayLayers: 1 };
  const texture = (texMismatched ? t.mismatchedDevice : t.device).createTexture({
    size: textureSize,
    format: 'rgba8unorm',
    usage: GPUTextureUsage.COPY_SRC | GPUTextureUsage.COPY_DST
  });
  t.trackForCleanup(texture);

  const isValid = !bufMismatched && !texMismatched;

  if (copyType === 'CopyB2T') {
    t.testCopyBufferToTexture({ buffer }, { texture }, textureSize, isValid);
  } else if (copyType === 'CopyT2B') {
    t.testCopyTextureToBuffer({ texture }, { buffer }, textureSize, isValid);
  }
});