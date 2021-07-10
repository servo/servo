/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/ export const description = `
copyBufferToBuffer tests.

Test Plan:
* Buffer is valid/invalid
  - the source buffer is invalid
  - the destination buffer is invalid
* Buffer usages
  - the source buffer is created without GPUBufferUsage::COPY_SRC
  - the destination buffer is created without GPUBufferUsage::COPY_DEST
* CopySize
  - copySize is not a multiple of 4
  - copySize is 0
* copy offsets
  - sourceOffset is not a multiple of 4
  - destinationOffset is not a multiple of 4
* Arthimetic overflow
  - (sourceOffset + copySize) is overflow
  - (destinationOffset + copySize) is overflow
* Out of bounds
  - (sourceOffset + copySize) > size of source buffer
  - (destinationOffset + copySize) > size of destination buffer
* Source buffer and destination buffer are the same buffer
`;
import { poptions, params } from '../../../common/framework/params_builder.js';
import { makeTestGroup } from '../../../common/framework/test_group.js';
import { kBufferUsages } from '../../capability_info.js';
import { kMaxSafeMultipleOf8 } from '../../util/math.js';

import { ValidationTest } from './validation_test.js';

class F extends ValidationTest {
  TestCopyBufferToBuffer(options) {
    const { srcBuffer, srcOffset, dstBuffer, dstOffset, copySize, isSuccess } = options;

    const commandEncoder = this.device.createCommandEncoder();
    commandEncoder.copyBufferToBuffer(srcBuffer, srcOffset, dstBuffer, dstOffset, copySize);

    this.expectValidationError(() => {
      commandEncoder.finish();
    }, !isSuccess);
  }
}

export const g = makeTestGroup(F);

g.test('copy_with_invalid_buffer').fn(async t => {
  const validBuffer = t.device.createBuffer({
    size: 16,
    usage: GPUBufferUsage.COPY_SRC | GPUBufferUsage.COPY_DST,
  });

  const errorBuffer = t.getErrorBuffer();

  t.TestCopyBufferToBuffer({
    srcBuffer: errorBuffer,
    srcOffset: 0,
    dstBuffer: validBuffer,
    dstOffset: 0,
    copySize: 8,
    isSuccess: false,
  });

  t.TestCopyBufferToBuffer({
    srcBuffer: validBuffer,
    srcOffset: 0,
    dstBuffer: errorBuffer,
    dstOffset: 0,
    copySize: 8,
    isSuccess: false,
  });
});

g.test('buffer_usage')
  .params(
    params()
      .combine(poptions('srcUsage', kBufferUsages))
      .combine(poptions('dstUsage', kBufferUsages))
  )
  .fn(async t => {
    const { srcUsage, dstUsage } = t.params;

    const srcBuffer = t.device.createBuffer({
      size: 16,
      usage: srcUsage,
    });

    const dstBuffer = t.device.createBuffer({
      size: 16,
      usage: dstUsage,
    });

    const isSuccess = srcUsage === GPUBufferUsage.COPY_SRC && dstUsage === GPUBufferUsage.COPY_DST;

    t.TestCopyBufferToBuffer({
      srcBuffer,
      srcOffset: 0,
      dstBuffer,
      dstOffset: 0,
      copySize: 8,
      isSuccess,
    });
  });

g.test('copy_size_alignment')
  .params([
    { copySize: 0, _isSuccess: true },
    { copySize: 2, _isSuccess: false },
    { copySize: 4, _isSuccess: true },
    { copySize: 5, _isSuccess: false },
    { copySize: 8, _isSuccess: true },
  ])
  .fn(async t => {
    const { copySize, _isSuccess: isSuccess } = t.params;

    const srcBuffer = t.device.createBuffer({
      size: 16,
      usage: GPUBufferUsage.COPY_SRC,
    });

    const dstBuffer = t.device.createBuffer({
      size: 16,
      usage: GPUBufferUsage.COPY_DST,
    });

    t.TestCopyBufferToBuffer({
      srcBuffer,
      srcOffset: 0,
      dstBuffer,
      dstOffset: 0,
      copySize,
      isSuccess,
    });
  });

g.test('copy_offset_alignment')
  .params([
    { srcOffset: 0, dstOffset: 0, _isSuccess: true },
    { srcOffset: 2, dstOffset: 0, _isSuccess: false },
    { srcOffset: 4, dstOffset: 0, _isSuccess: true },
    { srcOffset: 5, dstOffset: 0, _isSuccess: false },
    { srcOffset: 8, dstOffset: 0, _isSuccess: true },
    { srcOffset: 0, dstOffset: 2, _isSuccess: false },
    { srcOffset: 0, dstOffset: 4, _isSuccess: true },
    { srcOffset: 0, dstOffset: 5, _isSuccess: false },
    { srcOffset: 0, dstOffset: 8, _isSuccess: true },
    { srcOffset: 4, dstOffset: 4, _isSuccess: true },
  ])
  .fn(async t => {
    const { srcOffset, dstOffset, _isSuccess: isSuccess } = t.params;

    const srcBuffer = t.device.createBuffer({
      size: 16,
      usage: GPUBufferUsage.COPY_SRC,
    });

    const dstBuffer = t.device.createBuffer({
      size: 16,
      usage: GPUBufferUsage.COPY_DST,
    });

    t.TestCopyBufferToBuffer({
      srcBuffer,
      srcOffset,
      dstBuffer,
      dstOffset,
      copySize: 8,
      isSuccess,
    });
  });

g.test('copy_overflow')
  .params([
    { srcOffset: 0, dstOffset: 0, copySize: kMaxSafeMultipleOf8 },
    { srcOffset: 16, dstOffset: 0, copySize: kMaxSafeMultipleOf8 },
    { srcOffset: 0, dstOffset: 16, copySize: kMaxSafeMultipleOf8 },
    { srcOffset: kMaxSafeMultipleOf8, dstOffset: 0, copySize: 16 },
    { srcOffset: 0, dstOffset: kMaxSafeMultipleOf8, copySize: 16 },
    { srcOffset: kMaxSafeMultipleOf8, dstOffset: 0, copySize: kMaxSafeMultipleOf8 },
    { srcOffset: 0, dstOffset: kMaxSafeMultipleOf8, copySize: kMaxSafeMultipleOf8 },
    {
      srcOffset: kMaxSafeMultipleOf8,
      dstOffset: kMaxSafeMultipleOf8,
      copySize: kMaxSafeMultipleOf8,
    },
  ])
  .fn(async t => {
    const { srcOffset, dstOffset, copySize } = t.params;

    const srcBuffer = t.device.createBuffer({
      size: 16,
      usage: GPUBufferUsage.COPY_SRC,
    });

    const dstBuffer = t.device.createBuffer({
      size: 16,
      usage: GPUBufferUsage.COPY_DST,
    });

    t.TestCopyBufferToBuffer({
      srcBuffer,
      srcOffset,
      dstBuffer,
      dstOffset,
      copySize,
      isSuccess: false,
    });
  });

g.test('copy_out_of_bounds')
  .params([
    { srcOffset: 0, dstOffset: 0, copySize: 32, _isSuccess: true },
    { srcOffset: 0, dstOffset: 0, copySize: 36 },
    { srcOffset: 36, dstOffset: 0, copySize: 4 },
    { srcOffset: 0, dstOffset: 36, copySize: 4 },
    { srcOffset: 36, dstOffset: 0, copySize: 0 },
    { srcOffset: 0, dstOffset: 36, copySize: 0 },
    { srcOffset: 20, dstOffset: 0, copySize: 16 },
    { srcOffset: 20, dstOffset: 0, copySize: 12, _isSuccess: true },
    { srcOffset: 0, dstOffset: 20, copySize: 16 },
    { srcOffset: 0, dstOffset: 20, copySize: 12, _isSuccess: true },
  ])
  .fn(async t => {
    const { srcOffset, dstOffset, copySize, _isSuccess = false } = t.params;

    const srcBuffer = t.device.createBuffer({
      size: 32,
      usage: GPUBufferUsage.COPY_SRC,
    });

    const dstBuffer = t.device.createBuffer({
      size: 32,
      usage: GPUBufferUsage.COPY_DST,
    });

    t.TestCopyBufferToBuffer({
      srcBuffer,
      srcOffset,
      dstBuffer,
      dstOffset,
      copySize,
      isSuccess: _isSuccess,
    });
  });

g.test('copy_within_same_buffer')
  .params([
    { srcOffset: 0, dstOffset: 8, copySize: 4 },
    { srcOffset: 8, dstOffset: 0, copySize: 4 },
    { srcOffset: 0, dstOffset: 4, copySize: 8 },
    { srcOffset: 4, dstOffset: 0, copySize: 8 },
  ])
  .fn(async t => {
    const { srcOffset, dstOffset, copySize } = t.params;

    const buffer = t.device.createBuffer({
      size: 16,
      usage: GPUBufferUsage.COPY_SRC | GPUBufferUsage.COPY_DST,
    });

    t.TestCopyBufferToBuffer({
      srcBuffer: buffer,
      srcOffset,
      dstBuffer: buffer,
      dstOffset,
      copySize,
      isSuccess: false,
    });
  });
