// Copyright 2023 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.map
description: >
  Array.p.map behaves as expected on TypedArrays backed by resizable buffers.
includes: [compareArray.js, resizableArrayBufferUtils.js]
features: [resizable-arraybuffer]
---*/

for (let ctor of ctors) {
  const rab = CreateResizableArrayBuffer(4 * ctor.BYTES_PER_ELEMENT, 8 * ctor.BYTES_PER_ELEMENT);
  const fixedLength = new ctor(rab, 0, 4);
  const fixedLengthWithOffset = new ctor(rab, 2 * ctor.BYTES_PER_ELEMENT, 2);
  const lengthTracking = new ctor(rab, 0);
  const lengthTrackingWithOffset = new ctor(rab, 2 * ctor.BYTES_PER_ELEMENT);

  // Write some data into the array.
  const taWrite = new ctor(rab);
  for (let i = 0; i < taWrite.length; ++i) {
    taWrite[i] = MayNeedBigInt(taWrite, 2 * i);
  }

  // Orig. array: [0, 2, 4, 6]
  //              [0, 2, 4, 6] << fixedLength
  //                    [4, 6] << fixedLengthWithOffset
  //              [0, 2, 4, 6, ...] << lengthTracking
  //                    [4, 6, ...] << lengthTrackingWithOffset

  function MapGatherCompare(array) {
    const values = [];
    function GatherValues(n, ix) {
      assert.sameValue(ix, values.length);
      values.push(n);
      if (typeof n == 'bigint') {
        return n + 1n;
      }
      return n + 1;
    }
    const newValues = Array.prototype.map.call(array, GatherValues);
    for (let i = 0; i < values.length; ++i) {
      if (typeof values[i] == 'bigint') {
        assert.sameValue(values[i] + 1n, newValues[i]);
      } else {
        assert.sameValue(values[i] + 1, newValues[i]);
      }
    }
    return ToNumbers(values);
  }
  assert.compareArray(MapGatherCompare(fixedLength), [
    0,
    2,
    4,
    6
  ]);
  assert.compareArray(MapGatherCompare(fixedLengthWithOffset), [
    4,
    6
  ]);
  assert.compareArray(MapGatherCompare(lengthTracking), [
    0,
    2,
    4,
    6
  ]);
  assert.compareArray(MapGatherCompare(lengthTrackingWithOffset), [
    4,
    6
  ]);

  // Shrink so that fixed length TAs go out of bounds.
  rab.resize(3 * ctor.BYTES_PER_ELEMENT);

  // Orig. array: [0, 2, 4]
  //              [0, 2, 4, ...] << lengthTracking
  //                    [4, ...] << lengthTrackingWithOffset

  assert.compareArray(MapGatherCompare(fixedLength), []);
  assert.compareArray(MapGatherCompare(fixedLengthWithOffset), []);

  assert.compareArray(MapGatherCompare(lengthTracking), [
    0,
    2,
    4
  ]);
  assert.compareArray(MapGatherCompare(lengthTrackingWithOffset), [4]);

  // Shrink so that the TAs with offset go out of bounds.
  rab.resize(1 * ctor.BYTES_PER_ELEMENT);
  assert.compareArray(MapGatherCompare(fixedLength), []);
  assert.compareArray(MapGatherCompare(fixedLengthWithOffset), []);
  assert.compareArray(MapGatherCompare(lengthTrackingWithOffset), []);

  assert.compareArray(MapGatherCompare(lengthTracking), [0]);

  // Shrink to zero.
  rab.resize(0);
  assert.compareArray(MapGatherCompare(fixedLength), []);
  assert.compareArray(MapGatherCompare(fixedLengthWithOffset), []);
  assert.compareArray(MapGatherCompare(lengthTrackingWithOffset), []);

  assert.compareArray(MapGatherCompare(lengthTracking), []);

  // Grow so that all TAs are back in-bounds.
  rab.resize(6 * ctor.BYTES_PER_ELEMENT);
  for (let i = 0; i < 6; ++i) {
    taWrite[i] = MayNeedBigInt(taWrite, 2 * i);
  }

  // Orig. array: [0, 2, 4, 6, 8, 10]
  //              [0, 2, 4, 6] << fixedLength
  //                    [4, 6] << fixedLengthWithOffset
  //              [0, 2, 4, 6, 8, 10, ...] << lengthTracking
  //                    [4, 6, 8, 10, ...] << lengthTrackingWithOffset

  assert.compareArray(MapGatherCompare(fixedLength), [
    0,
    2,
    4,
    6
  ]);
  assert.compareArray(MapGatherCompare(fixedLengthWithOffset), [
    4,
    6
  ]);
  assert.compareArray(MapGatherCompare(lengthTracking), [
    0,
    2,
    4,
    6,
    8,
    10
  ]);
  assert.compareArray(MapGatherCompare(lengthTrackingWithOffset), [
    4,
    6,
    8,
    10
  ]);
}
