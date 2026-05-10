// Copyright 2023 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-%typedarray%.prototype.set
description: >
  TypedArray.p.set behaves correctly on TypedArrays backed by resizable buffers.
includes: [compareArray.js, resizableArrayBufferUtils.js]
features: [resizable-arraybuffer]
---*/

function IsBigIntTypedArray(ta) {
  return ta instanceof BigInt64Array || ta instanceof BigUint64Array;
}

function SetNumOrBigInt(target, source, offset) {
  if (IsBigIntTypedArray(target)) {
    const bigIntSource = [];
    for (const s of source) {
      bigIntSource.push(BigInt(s));
    }
    source = bigIntSource;
  }
  if (offset == undefined) {
    return target.set(source);
  }
  return target.set(source, offset);
}

for (let targetIsResizable of [
    false,
    true
  ]) {
  for (let targetCtor of ctors) {
    for (let sourceCtor of ctors) {
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

      const targetAb = targetIsResizable ? new ArrayBuffer(6 * targetCtor.BYTES_PER_ELEMENT) : new ArrayBuffer(6 * targetCtor.BYTES_PER_ELEMENT, { maxByteLength: 8 * targetCtor.BYTES_PER_ELEMENT });
      const target = new targetCtor(targetAb);
      if (IsBigIntTypedArray(target) != IsBigIntTypedArray(taFull)) {
        // Can't mix BigInt and non-BigInt types.
        continue;
      }
      SetNumOrBigInt(target, fixedLength);
      assert.compareArray(ToNumbers(target), [
        1,
        2,
        3,
        4,
        0,
        0
      ]);
      SetNumOrBigInt(target, fixedLengthWithOffset);
      assert.compareArray(ToNumbers(target), [
        3,
        4,
        3,
        4,
        0,
        0
      ]);
      SetNumOrBigInt(target, lengthTracking, 1);
      assert.compareArray(ToNumbers(target), [
        3,
        1,
        2,
        3,
        4,
        0
      ]);
      SetNumOrBigInt(target, lengthTrackingWithOffset, 1);
      assert.compareArray(ToNumbers(target), [
        3,
        3,
        4,
        3,
        4,
        0
      ]);

      // Shrink so that fixed length TAs go out of bounds.
      rab.resize(3 * sourceCtor.BYTES_PER_ELEMENT);

      // Orig. array: [1, 2, 3]
      //              [1, 2, 3, ...] << lengthTracking
      //                    [3, ...] << lengthTrackingWithOffset

      assert.throws(TypeError, () => {
        SetNumOrBigInt(target, fixedLength);
      });
      assert.throws(TypeError, () => {
        SetNumOrBigInt(target, fixedLengthWithOffset);
      });
      assert.compareArray(ToNumbers(target), [
        3,
        3,
        4,
        3,
        4,
        0
      ]);
      SetNumOrBigInt(target, lengthTracking);
      assert.compareArray(ToNumbers(target), [
        1,
        2,
        3,
        3,
        4,
        0
      ]);
      SetNumOrBigInt(target, lengthTrackingWithOffset);
      assert.compareArray(ToNumbers(target), [
        3,
        2,
        3,
        3,
        4,
        0
      ]);

      // Shrink so that the TAs with offset go out of bounds.
      rab.resize(1 * sourceCtor.BYTES_PER_ELEMENT);
      assert.throws(TypeError, () => {
        SetNumOrBigInt(target, fixedLength);
      });
      assert.throws(TypeError, () => {
        SetNumOrBigInt(target, fixedLengthWithOffset);
      });
      assert.throws(TypeError, () => {
        SetNumOrBigInt(target, lengthTrackingWithOffset);
      });
      SetNumOrBigInt(target, lengthTracking, 3);
      assert.compareArray(ToNumbers(target), [
        3,
        2,
        3,
        1,
        4,
        0
      ]);

      // Shrink to zero.
      rab.resize(0);
      assert.throws(TypeError, () => {
        SetNumOrBigInt(target, fixedLength);
      });
      assert.throws(TypeError, () => {
        SetNumOrBigInt(target, fixedLengthWithOffset);
      });
      assert.throws(TypeError, () => {
        SetNumOrBigInt(target, lengthTrackingWithOffset);
      });
      SetNumOrBigInt(target, lengthTracking, 4);
      assert.compareArray(ToNumbers(target), [
        3,
        2,
        3,
        1,
        4,
        0
      ]);

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

      SetNumOrBigInt(target, fixedLength);
      assert.compareArray(ToNumbers(target), [
        1,
        2,
        3,
        4,
        4,
        0
      ]);
      SetNumOrBigInt(target, fixedLengthWithOffset);
      assert.compareArray(ToNumbers(target), [
        3,
        4,
        3,
        4,
        4,
        0
      ]);
      SetNumOrBigInt(target, lengthTracking, 0);
      assert.compareArray(ToNumbers(target), [
        1,
        2,
        3,
        4,
        5,
        6
      ]);
      SetNumOrBigInt(target, lengthTrackingWithOffset, 1);
      assert.compareArray(ToNumbers(target), [
        1,
        3,
        4,
        5,
        6,
        6
      ]);
    }
  }
}
