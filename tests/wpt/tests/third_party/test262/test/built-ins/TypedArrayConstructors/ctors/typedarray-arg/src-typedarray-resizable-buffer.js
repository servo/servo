// Copyright 2023 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-initializetypedarrayfromtypedarray
description: >
  Initializing a TypedArray from another TypedArray that is backed by a
  resizable buffer
includes: [compareArray.js, resizableArrayBufferUtils.js]
features: [resizable-arraybuffer]
---*/

function IsBigIntTypedArray(ta) {
  return ta instanceof BigInt64Array || ta instanceof BigUint64Array;
}

function AllBigIntMatchedCtorCombinations(test) {
  for (let targetCtor of ctors) {
    for (let sourceCtor of ctors) {
      if (IsBigIntTypedArray(new targetCtor()) != IsBigIntTypedArray(new sourceCtor())) {
        continue;
      }
      test(targetCtor, sourceCtor);
    }
  }
}

AllBigIntMatchedCtorCombinations((targetCtor, sourceCtor) => {
  const rab = CreateResizableArrayBuffer(4 * sourceCtor.BYTES_PER_ELEMENT, 8 * sourceCtor.BYTES_PER_ELEMENT);
  const fixedLength = new sourceCtor(rab, 0, 4);
  const fixedLengthWithOffset = new sourceCtor(rab, 2 * sourceCtor.BYTES_PER_ELEMENT, 2);
  const lengthTracking = new sourceCtor(rab, 0);
  const lengthTrackingWithOffset = new sourceCtor(rab, 2 * sourceCtor.BYTES_PER_ELEMENT);

  // Write some data into the array.
  const taFull = new sourceCtor(rab);
  for (let i = 0; i < 4; ++i) {
    taFull[i] = MayNeedBigInt(taFull, i + 1);
  }

  // Orig. array: [1, 2, 3, 4]
  //              [1, 2, 3, 4] << fixedLength
  //                    [3, 4] << fixedLengthWithOffset
  //              [1, 2, 3, 4, ...] << lengthTracking
  //                    [3, 4, ...] << lengthTrackingWithOffset

  assert.compareArray(ToNumbers(new targetCtor(fixedLength)), [
    1,
    2,
    3,
    4
  ]);
  assert.compareArray(ToNumbers(new targetCtor(fixedLengthWithOffset)), [
    3,
    4
  ]);
  assert.compareArray(ToNumbers(new targetCtor(lengthTracking)), [
    1,
    2,
    3,
    4
  ]);
  assert.compareArray(ToNumbers(new targetCtor(lengthTrackingWithOffset)), [
    3,
    4
  ]);

  // Shrink so that fixed length TAs go out of bounds.
  rab.resize(3 * sourceCtor.BYTES_PER_ELEMENT);

  // Orig. array: [1, 2, 3]
  //              [1, 2, 3, ...] << lengthTracking
  //                    [3, ...] << lengthTrackingWithOffset

  assert.throws(TypeError, () => {
    new targetCtor(fixedLength);
  });
  assert.throws(TypeError, () => {
    new targetCtor(fixedLengthWithOffset);
  });
  assert.compareArray(ToNumbers(new targetCtor(lengthTracking)), [
    1,
    2,
    3
  ]);
  assert.compareArray(ToNumbers(new targetCtor(lengthTrackingWithOffset)), [3]);

  // Shrink so that the TAs with offset go out of bounds.
  rab.resize(1 * sourceCtor.BYTES_PER_ELEMENT);
  assert.throws(TypeError, () => {
    new targetCtor(fixedLength);
  });
  assert.throws(TypeError, () => {
    new targetCtor(fixedLengthWithOffset);
  });
  assert.compareArray(ToNumbers(new targetCtor(lengthTracking)), [1]);
  assert.throws(TypeError, () => {
    new targetCtor(lengthTrackingWithOffset);
  });

  // Shrink to zero.
  rab.resize(0);
  assert.throws(TypeError, () => {
    new targetCtor(fixedLength);
  });
  assert.throws(TypeError, () => {
    new targetCtor(fixedLengthWithOffset);
  });
  assert.compareArray(ToNumbers(new targetCtor(lengthTracking)), []);
  assert.throws(TypeError, () => {
    new targetCtor(lengthTrackingWithOffset);
  });

  // Grow so that all TAs are back in-bounds.
  rab.resize(6 * sourceCtor.BYTES_PER_ELEMENT);
  for (let i = 0; i < 6; ++i) {
    taFull[i] = MayNeedBigInt(taFull, i + 1);
  }

  // Orig. array: [1, 2, 3, 4, 5, 6]
  //              [1, 2, 3, 4] << fixedLength
  //                    [3, 4] << fixedLengthWithOffset
  //              [1, 2, 3, 4, 5, 6, ...] << lengthTracking
  //                    [3, 4, 5, 6, ...] << lengthTrackingWithOffset

  assert.compareArray(ToNumbers(new targetCtor(fixedLength)), [
    1,
    2,
    3,
    4
  ]);
  assert.compareArray(ToNumbers(new targetCtor(fixedLengthWithOffset)), [
    3,
    4
  ]);
  assert.compareArray(ToNumbers(new targetCtor(lengthTracking)), [
    1,
    2,
    3,
    4,
    5,
    6
  ]);
  assert.compareArray(ToNumbers(new targetCtor(lengthTrackingWithOffset)), [
    3,
    4,
    5,
    6
  ]);
});
