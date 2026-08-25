/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
setImmediates validation tests.
TODO(#4297): enable Float16Array
`;import { makeTestGroup } from '../../../../../common/framework/test_group.js';
import { getGPU } from '../../../../../common/util/navigator_gpu.js';
import {
  kTypedArrayBufferViews,
  kTypedArrayBufferViewKeys,
  supportsImmediateData } from
'../../../../../common/util/util.js';
import { AllFeaturesMaxLimitsGPUTest } from '../../../../gpu_test.js';
import { kProgrammableEncoderTypes } from '../../../../util/command_buffer_maker.js';

class SetImmediatesTest extends AllFeaturesMaxLimitsGPUTest {
  async init() {
    await super.init();
    if (!supportsImmediateData(getGPU(this.rec))) {
      this.skip('setImmediates not supported');
    }
  }
}

export const g = makeTestGroup(SetImmediatesTest);

g.test('alignment').
desc('Tests that rangeOffset and contentSize must align to 4 bytes.').
params((u) =>
u //
.combine('encoderType', kProgrammableEncoderTypes).
combine('arrayType', kTypedArrayBufferViewKeys).
filter((p) => p.arrayType !== 'Float16Array').
combineWithParams([
// control case: rangeOffset 4 is aligned. contentByteSize 8 is aligned.
{ rangeOffset: 4, contentByteSize: 8 },
// rangeOffset 6 is unaligned (6 % 4 !== 0).
{ rangeOffset: 6, contentByteSize: 8 },
// contentByteSize 10 is unaligned (10 % 4 !== 0).
// Note: This case will be skipped for types with element size > 2 (e.g. Uint32, Uint64)
// because they cannot form a 10-byte array.
{ rangeOffset: 4, contentByteSize: 10 }]
).
filter(({ arrayType, contentByteSize }) => {
  // Skip if the contentByteSize is not a multiple of the element size.
  // For example, we can't have 10 bytes if the element size is 4 or 8 bytes.
  const arrayConstructor = kTypedArrayBufferViews[arrayType];
  return contentByteSize % arrayConstructor.BYTES_PER_ELEMENT === 0;
})
).
fn((t) => {
  const { encoderType, arrayType, rangeOffset, contentByteSize } = t.params;
  const arrayBufferType = kTypedArrayBufferViews[arrayType];
  const elementSize = arrayBufferType.BYTES_PER_ELEMENT;
  const elementCount = contentByteSize / elementSize;

  const isRangeOffsetAligned = rangeOffset % 4 === 0;
  const isContentSizeAligned = contentByteSize % 4 === 0;

  const { encoder, validateFinish } = t.createEncoder(encoderType);
  const data = new arrayBufferType(elementCount);

  t.shouldThrow(isContentSizeAligned ? false : 'RangeError', () => {
    encoder.setImmediates(rangeOffset, data, 0, elementCount);
  });

  validateFinish(isRangeOffsetAligned);
});

g.test('overflow').
desc(
  `
    Tests that rangeOffset + contentSize or dataOffset + size is handled correctly if it exceeds limits.
    `
).
params((u) =>
u //
.combine('encoderType', kProgrammableEncoderTypes).
combine('arrayType', kTypedArrayBufferViewKeys).
filter((p) => p.arrayType !== 'Float16Array').
combineWithParams([
// control case
{ rangeOffset: 0, dataOffset: 0, elementCount: 4, _expectedError: null },
// elementCount 0
{ rangeOffset: 0, dataOffset: 0, elementCount: 0, _expectedError: null },
// rangeOffset + contentSize overflows
{
  rangeOffset: 2 ** 31 - 8,
  dataOffset: 0,
  elementCount: 4,
  _expectedError: 'validation'
},
{
  rangeOffset: 2 ** 32 - 8,
  dataOffset: 0,
  elementCount: 4,
  _expectedError: 'validation'
},
// dataOffset + size overflows
{
  rangeOffset: 0,
  dataOffset: 2 ** 31 - 1,
  elementCount: 4,
  _expectedError: 'RangeError'
},
{
  rangeOffset: 0,
  dataOffset: 2 ** 32 - 1,
  elementCount: 4,
  _expectedError: 'RangeError'
}]
)
).
fn((t) => {
  const { encoderType, arrayType, rangeOffset, dataOffset, elementCount, _expectedError } =
  t.params;
  const arrayBufferType = kTypedArrayBufferViews[arrayType];

  const { encoder, validateFinish } = t.createEncoder(encoderType);
  const data = new arrayBufferType(elementCount);

  const doSetImmediates = () => {
    encoder.setImmediates(rangeOffset, data, dataOffset, elementCount);
  };

  if (_expectedError === 'RangeError') {
    t.shouldThrow('RangeError', doSetImmediates);
  } else {
    doSetImmediates();
    validateFinish(_expectedError === null);
  }
});

g.test('out_of_bounds').
desc(
  `
    Tests that rangeOffset + contentSize is greater than maxImmediateSize (Validation Error)
    and contentSize is larger than data size (RangeError).
    `
).
params((u) =>
u //
.combine('encoderType', kProgrammableEncoderTypes).
combine('arrayType', kTypedArrayBufferViewKeys).
filter((p) => p.arrayType !== 'Float16Array').
combineWithParams([
// control case
{ rangeOffsetDelta: 0, dataLengthDelta: 0 },
// rangeOffset + contentSize > maxImmediateSize
{ rangeOffsetDelta: 4, dataLengthDelta: 0 },
// dataOffset + size > data.length
{ rangeOffsetDelta: 0, dataLengthDelta: -1 }]
)
).
fn((t) => {
  const { encoderType, arrayType, rangeOffsetDelta, dataLengthDelta } = t.params;
  const arrayBufferType = kTypedArrayBufferViews[arrayType];
  const elementSize = arrayBufferType.BYTES_PER_ELEMENT;

  const maxImmediateSize = t.device.limits.maxImmediateSize;
  if (maxImmediateSize === undefined) {
    t.skip('maxImmediateSize not found');
  }

  // We want contentByteSize to be aligned to 4 bytes to avoid alignment errors.
  // We use 8 bytes to cover all types including BigUint64 (8 bytes).
  const elementCount = elementSize >= 8 ? 1 : 8 / elementSize;
  const contentByteSize = elementCount * elementSize;

  const rangeOffset = maxImmediateSize - contentByteSize + rangeOffsetDelta;
  const dataLength = elementCount + dataLengthDelta;

  const data = new arrayBufferType(dataLength);

  const { encoder, validateFinish } = t.createEncoder(encoderType);

  const rangeOverLimit = rangeOffset + contentByteSize > maxImmediateSize;
  const dataOverLimit = elementCount > dataLength;

  t.shouldThrow(dataOverLimit ? 'RangeError' : false, () => {
    encoder.setImmediates(rangeOffset, data, 0, elementCount);
  });

  if (!dataOverLimit) {
    validateFinish(!rangeOverLimit);
  }
});