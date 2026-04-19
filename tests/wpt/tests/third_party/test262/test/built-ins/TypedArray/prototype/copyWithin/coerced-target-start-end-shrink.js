// Copyright 2023 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.copywithin
description: >
  TypedArray.p.copyWithin behaves correctly when argument coercion shrinks the receiver
includes: [compareArray.js, resizableArrayBufferUtils.js]
features: [resizable-arraybuffer]
---*/

const fillWithIndexes = (ta, length) => {
  for (let i = 0; i < length; ++i) {
    ta[i] = MayNeedBigInt(ta, i);
  }
  return ta;
};

for (let ctor of ctors) {
  const rab = CreateResizableArrayBuffer(4 * ctor.BYTES_PER_ELEMENT, 8 * ctor.BYTES_PER_ELEMENT);
  const fixedLength = new ctor(rab, 0, 4);
  const evil = () => {
    rab.resize(2 * ctor.BYTES_PER_ELEMENT);
    return 2;
  };
  assert.throws(TypeError, () => {
    fixedLength.copyWithin({ valueOf: evil }, 0, 1);
  }, ctor.name + " evil target.");
  rab.resize(4 * ctor.BYTES_PER_ELEMENT);
  assert.throws(TypeError, () => {
    fixedLength.copyWithin(0, { valueOf: evil }, 3);
  }, ctor.name + " evil start.");
  rab.resize(4 * ctor.BYTES_PER_ELEMENT);
  assert.throws(TypeError, () => {
    fixedLength.copyWithin(0, 1, { valueOf: evil });
  }, ctor.name + " evil end.");
}

for (let ctor of ctors) {
  const rab = CreateResizableArrayBuffer(4 * ctor.BYTES_PER_ELEMENT, 8 * ctor.BYTES_PER_ELEMENT);
  const lengthTracking = fillWithIndexes(new ctor(rab), 4);
  // [0, 1, 2,] 3]
  //        <=--> dest
  //  <=-> src
  const evil = () => {
    rab.resize(3 * ctor.BYTES_PER_ELEMENT);
    return 2;
  };
  lengthTracking.copyWithin({ valueOf: evil }, 0);
  assert.compareArray(ToNumbers(lengthTracking), [0, 1, 0],
    ctor.name + " truncated copy forward.");
}

for (let ctor of ctors) {
  const rab = CreateResizableArrayBuffer(4 * ctor.BYTES_PER_ELEMENT, 8 * ctor.BYTES_PER_ELEMENT);
  const lengthTracking = fillWithIndexes(new ctor(rab), 4);
  // [0, 1, 2,] 3]
  //        <=--> src
  //  <=-> dest
  const evil = () => {
    rab.resize(3 * ctor.BYTES_PER_ELEMENT);
    return 2;
  };
  lengthTracking.copyWithin(0, { valueOf: evil });
  assert.compareArray(ToNumbers(lengthTracking), [2, 1, 2],
    ctor.name + " truncated copy backward.");
}

for (let ctor of ctors) {
  const rab = CreateResizableArrayBuffer(4 * ctor.BYTES_PER_ELEMENT, 8 * ctor.BYTES_PER_ELEMENT);
  const lengthTracking = fillWithIndexes(new ctor(rab), 4);
  // [0, 1, 2,] 3]
  //        <=--> dest
  //     <=-> src
  const evil = () => {
    rab.resize(3 * ctor.BYTES_PER_ELEMENT);
    return 2;
  };
  lengthTracking.copyWithin({ valueOf: evil }, 1);
  assert.compareArray(ToNumbers(lengthTracking), [0, 1, 1],
    ctor.name + " truncated overlapping copy forward.");
}

for (let ctor of ctors) {
  const rab = CreateResizableArrayBuffer(4 * ctor.BYTES_PER_ELEMENT, 8 * ctor.BYTES_PER_ELEMENT);
  const lengthTracking = fillWithIndexes(new ctor(rab), 4);
  // [0, 1, 2,] 3]
  //        <=--> src
  //     <=-> dest
  const evil = () => {
    rab.resize(3 * ctor.BYTES_PER_ELEMENT);
    return 2;
  };
  lengthTracking.copyWithin(1, { valueOf: evil });
  assert.compareArray(ToNumbers(lengthTracking), [0, 2, 2],
    ctor.name + " truncated overlapping copy backward.");
}
