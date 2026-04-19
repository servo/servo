// Copyright 2023 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-%typedarray%.prototype.subarray
description: >
  TypedArray.p.subarray behaves correctly on TypedArrays backed by resizable
  buffers that are shrunk by argument coercion.
includes: [compareArray.js, resizableArrayBufferUtils.js]
features: [resizable-arraybuffer]
---*/

// Orig. array: [0, 2, 4, 6]
//              [0, 2, 4, 6] << fixedLength
//              [0, 2, 4, 6, ...] << lengthTracking


// Fixed-length TA + first parameter conversion shrinks. The old length is
// used in the length computation, and the subarray construction fails.
for (let ctor of ctors) {
  const rab = CreateRabForTest(ctor);
  const fixedLength = new ctor(rab, 0, 4);
  let evil = {
    valueOf: () => {
      rab.resize(2 * ctor.BYTES_PER_ELEMENT);
      return 0;
    }
  };
  assert.throws(RangeError, () => {
    fixedLength.subarray(evil);
  });
}

// Like the previous test, but now we construct a smaller subarray and it
// succeeds.
for (let ctor of ctors) {
  const rab = CreateRabForTest(ctor);
  const fixedLength = new ctor(rab, 0, 4);
  let evil = {
    valueOf: () => {
      rab.resize(2 * ctor.BYTES_PER_ELEMENT);
      return 0;
    }
  };
  assert.compareArray(ToNumbers(fixedLength.subarray(evil, 1)), [0]);
}

// As above but with the second parameter conversion shrinking the buffer.
for (let ctor of ctors) {
  const rab = CreateRabForTest(ctor);
  const fixedLength = new ctor(rab, 0, 4);
  let evil = {
    valueOf: () => {
      rab.resize(2 * ctor.BYTES_PER_ELEMENT);
      return 1;
    }
  };
  assert.compareArray(ToNumbers(fixedLength.subarray(0,evil)), [0]);
}

// Fixed-length TA + second parameter conversion shrinks. The old length is
// used in the length computation, and the subarray construction fails.
for (let ctor of ctors) {
  const rab = CreateRabForTest(ctor);
  const fixedLength = new ctor(rab, 0, 4);
  let evil = {
    valueOf: () => {
      rab.resize(2 * ctor.BYTES_PER_ELEMENT);
      return 3;
    }
  };
  assert.throws(RangeError, () => {
    fixedLength.subarray(0, evil);
  });
}

// Like the previous test, but now we construct a smaller subarray and it
// succeeds.
for (let ctor of ctors) {
  const rab = CreateRabForTest(ctor);
  const fixedLength = new ctor(rab, 0, 4);
  let evil = {
    valueOf: () => {
      rab.resize(2 * ctor.BYTES_PER_ELEMENT);
      return 1;
    }
  };
  assert.compareArray(ToNumbers(fixedLength.subarray(0, evil)), [0]);
}

// Shrinking + fixed-length TA, subarray construction succeeds even though the
// TA goes OOB.
for (let ctor of ctors) {
  const rab = CreateRabForTest(ctor);
  const fixedLength = new ctor(rab, 0, 4);
  const evil = {
    valueOf: () => {
      rab.resize(2 * ctor.BYTES_PER_ELEMENT);
      return 0;
    }
  };
  assert.compareArray(ToNumbers(fixedLength.subarray(evil, 1)), [0]);
}

// As above but with the second parameter conversion shrinking the buffer.
for (let ctor of ctors) {
  const rab = CreateRabForTest(ctor);
  const fixedLength = new ctor(rab, 0, 4);
  const evil = {
    valueOf: () => {
      rab.resize(2 * ctor.BYTES_PER_ELEMENT);
      return 1;
    }
  };
  assert.compareArray(ToNumbers(fixedLength.subarray(0,evil)), [0]);
}

// Length-tracking TA + first parameter conversion shrinks. The old length is
// used in the length computation, and the subarray construction fails.
for (let ctor of ctors) {
  const rab = CreateRabForTest(ctor);
  const lengthTracking = new ctor(rab);
  let evil = {
    valueOf: () => {
      rab.resize(2 * ctor.BYTES_PER_ELEMENT);
      return 0;
    }
  };
  assert.throws(RangeError, () => {
    lengthTracking.subarray(evil, lengthTracking.length);
  });
}

// Like the previous test, but now we construct a smaller subarray and it
// succeeds.
for (let ctor of ctors) {
  const rab = CreateRabForTest(ctor);
  const lengthTracking = new ctor(rab);
  let evil = {
    valueOf: () => {
      rab.resize(2 * ctor.BYTES_PER_ELEMENT);
      return 0;
    }
  };
  assert.compareArray(ToNumbers(lengthTracking.subarray(evil, 1)), [0]);
}

// As above but with the second parameter conversion shrinking the buffer.
for (let ctor of ctors) {
  const rab = CreateRabForTest(ctor);
  const lengthTracking = new ctor(rab);
  let evil = {
    valueOf: () => {
      rab.resize(2 * ctor.BYTES_PER_ELEMENT);
      return 1;
    }
  };
  assert.compareArray(ToNumbers(lengthTracking.subarray(0,evil)), [0]);
}

// Length-tracking TA + first parameter conversion shrinks. The second
// parameter is negative -> the relative index is not recomputed, and the
// subarray construction fails.
for (let ctor of ctors) {
  const rab = CreateRabForTest(ctor);
  const lengthTracking = new ctor(rab);
  let evil = {
    valueOf: () => {
      rab.resize(2 * ctor.BYTES_PER_ELEMENT);
      return 0;
    }
  };
  assert.throws(RangeError, () => {
    lengthTracking.subarray(evil, -1);
  });
}

// Length-tracking TA + second parameter conversion shrinks. The second
// parameter is too large -> the subarray construction fails.
for (let ctor of ctors) {
  const rab = CreateRabForTest(ctor);
  const lengthTracking = new ctor(rab);
  let evil = {
    valueOf: () => {
      rab.resize(2 * ctor.BYTES_PER_ELEMENT);
      return 3;
    }
  };
  assert.throws(RangeError, () => {
    lengthTracking.subarray(0, evil);
  });
}
