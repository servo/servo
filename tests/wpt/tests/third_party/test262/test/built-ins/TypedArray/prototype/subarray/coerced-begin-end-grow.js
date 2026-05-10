// Copyright 2023 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-%typedarray%.prototype.subarray
description: >
  TypedArray.p.subarray behaves correctly on TypedArrays backed by resizable
  buffers that are grown by argument coercion.
includes: [compareArray.js, resizableArrayBufferUtils.js]
features: [resizable-arraybuffer]
---*/

// Orig. array: [0, 2, 4, 6]
//              [0, 2, 4, 6] << fixedLength
//              [0, 2, 4, 6, ...] << lengthTracking

// Growing a fixed length TA back in bounds.
for (let ctor of ctors) {
  const rab = CreateRabForTest(ctor);
  const fixedLength = new ctor(rab, 0, 4);
  // Make `fixedLength` OOB.
  rab.resize(2 * ctor.BYTES_PER_ELEMENT);
  const evil = {
    valueOf: () => {
      rab.resize(4 * ctor.BYTES_PER_ELEMENT);
      return 0;
    }
  };
  // The length computation is done before parameter conversion. At that
  // point, the length is 0, since the TA is OOB.
  assert.compareArray(ToNumbers(fixedLength.subarray(evil, 1)), []);
}

// As above but with the second parameter conversion growing the buffer.
for (let ctor of ctors) {
  const rab = CreateRabForTest(ctor);
  const fixedLength = new ctor(rab, 0, 4);
  // Make `fixedLength` OOB.
  rab.resize(2 * ctor.BYTES_PER_ELEMENT);
  const evil = {
    valueOf: () => {
      rab.resize(4 * ctor.BYTES_PER_ELEMENT);
      return 1;
    }
  };
  // The length computation is done before parameter conversion. At that
  // point, the length is 0, since the TA is OOB.
  assert.compareArray(ToNumbers(fixedLength.subarray(0, evil)), []);
}


// Growing + fixed-length TA. Growing won't affect anything.
for (let ctor of ctors) {
  const rab = CreateRabForTest(ctor);
  const fixedLength = new ctor(rab, 0, 4);
  const evil = {
    valueOf: () => {
      rab.resize(6 * ctor.BYTES_PER_ELEMENT);
      return 0;
    }
  };
  assert.compareArray(ToNumbers(fixedLength.subarray(evil)), [
    0,
    2,
    4,
    6
  ]);
}

// As above but with the second parameter conversion growing the buffer.
for (let ctor of ctors) {
  const rab = CreateRabForTest(ctor);
  const fixedLength = new ctor(rab, 0, 4);
  const evil = {
    valueOf: () => {
      rab.resize(6 * ctor.BYTES_PER_ELEMENT);
      return 4;
    }
  };
  assert.compareArray(ToNumbers(fixedLength.subarray(0, evil)), [
    0,
    2,
    4,
    6
  ]);
}

// Growing + length-tracking TA. The length computation is done with the
// original length.
for (let ctor of ctors) {
  const rab = CreateRabForTest(ctor);
  const lengthTracking = new ctor(rab, 0);
  const evil = {
    valueOf: () => {
      rab.resize(6 * ctor.BYTES_PER_ELEMENT);
      return 0;
    }
  };
  assert.compareArray(
    ToNumbers(lengthTracking.subarray(evil, lengthTracking.length)), [
    0,
    2,
    4,
    6
  ]);
}
