// Copyright 2023 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-%typedarray%.prototype.lastindexof
description: >
  TypedArray.p.lastIndexOf behaves correctly on TypedArrays backed by resizable
  buffers that are shrunk by argument coercion.
includes: [resizableArrayBufferUtils.js]
features: [resizable-arraybuffer]
---*/

// Shrinking + fixed-length TA.
for (let ctor of ctors) {
  const rab = CreateResizableArrayBuffer(4 * ctor.BYTES_PER_ELEMENT, 8 * ctor.BYTES_PER_ELEMENT);
  const fixedLength = new ctor(rab, 0, 4);
  let evil = {
    valueOf: () => {
      rab.resize(2 * ctor.BYTES_PER_ELEMENT);
      return 2;
    }
  };
  let n0 = MayNeedBigInt(fixedLength, 0);
  assert.sameValue(fixedLength.lastIndexOf(n0), 3);
  // The TA is OOB so lastIndexOf returns -1.
  assert.sameValue(fixedLength.lastIndexOf(n0, evil), -1);
}
for (let ctor of ctors) {
  const rab = CreateResizableArrayBuffer(4 * ctor.BYTES_PER_ELEMENT, 8 * ctor.BYTES_PER_ELEMENT);
  const fixedLength = new ctor(rab, 0, 4);
  let evil = {
    valueOf: () => {
      rab.resize(2 * ctor.BYTES_PER_ELEMENT);
      return 2;
    }
  };
  let n0 = MayNeedBigInt(fixedLength, 0);
  assert.sameValue(fixedLength.lastIndexOf(n0), 3);
  // The TA is OOB so lastIndexOf returns -1, also for undefined).
  assert.sameValue(fixedLength.lastIndexOf(undefined, evil), -1);
}

// Shrinking + length-tracking TA.
for (let ctor of ctors) {
  const rab = CreateResizableArrayBuffer(4 * ctor.BYTES_PER_ELEMENT, 8 * ctor.BYTES_PER_ELEMENT);
  const lengthTracking = new ctor(rab);
  for (let i = 0; i < 4; ++i) {
    lengthTracking[i] = MayNeedBigInt(lengthTracking, i);
  }
  let evil = {
    valueOf: () => {
      rab.resize(2 * ctor.BYTES_PER_ELEMENT);
      return 2;
    }
  };
  let n2 = MayNeedBigInt(lengthTracking, 2);
  assert.sameValue(lengthTracking.lastIndexOf(n2), 2);
  // 2 no longer found.
  assert.sameValue(lengthTracking.lastIndexOf(n2, evil), -1);
}
