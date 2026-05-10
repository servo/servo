// Copyright 2023 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.indexof
description: >
  Array.p.indexOf behaves correctly when the backing resizable buffer is grown
  during argument coercion.
includes: [resizableArrayBufferUtils.js]
features: [resizable-arraybuffer]
---*/

// Growing + length-tracking TA.
for (let ctor of ctors) {
  const rab = CreateResizableArrayBuffer(4 * ctor.BYTES_PER_ELEMENT, 8 * ctor.BYTES_PER_ELEMENT);
  const lengthTracking = new ctor(rab);
  for (let i = 0; i < 4; ++i) {
    lengthTracking[i] = MayNeedBigInt(lengthTracking, 1);
  }
  let evil = {
    valueOf: () => {
      rab.resize(6 * ctor.BYTES_PER_ELEMENT);
      return 0;
    }
  };
  let n0 = MayNeedBigInt(lengthTracking, 0);
  assert.sameValue(Array.prototype.indexOf.call(lengthTracking, n0), -1);
  // The TA grew but we only look at the data until the original length.
  assert.sameValue(Array.prototype.indexOf.call(lengthTracking, n0, evil), -1);
}

// Growing + length-tracking TA, index conversion.
for (let ctor of ctors) {
  const rab = CreateResizableArrayBuffer(4 * ctor.BYTES_PER_ELEMENT, 8 * ctor.BYTES_PER_ELEMENT);
  const lengthTracking = new ctor(rab);
  lengthTracking[0] = MayNeedBigInt(lengthTracking, 1);
  let evil = {
    valueOf: () => {
      rab.resize(6 * ctor.BYTES_PER_ELEMENT);
      return -4;
    }
  };
  let n1 = MayNeedBigInt(lengthTracking, 1);
  assert.sameValue(Array.prototype.indexOf.call(lengthTracking, n1, -4), 0);
  // The TA grew but the start index conversion is done based on the original
  // length.
  assert.sameValue(Array.prototype.indexOf.call(lengthTracking, n1, evil), 0);
}
