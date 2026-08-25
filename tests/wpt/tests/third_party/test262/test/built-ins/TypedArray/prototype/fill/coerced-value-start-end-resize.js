// Copyright 2023 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-%typedarray%.prototype.fill
description: >
  TypedArray.p.fill behaves correctly on receivers backed by a resizable
  buffer that's resized during argument coercion
includes: [resizableArrayBufferUtils.js]
features: [resizable-arraybuffer]
---*/

function TypedArrayFillHelper(ta, n, start, end) {
  if (ta instanceof BigInt64Array || ta instanceof BigUint64Array) {
    ta.fill(BigInt(n), start, end);
  } else {
    ta.fill(n, start, end);
  }
}

for (let ctor of ctors) {
  const rab = CreateResizableArrayBuffer(4 * ctor.BYTES_PER_ELEMENT, 8 * ctor.BYTES_PER_ELEMENT);
  const fixedLength = new ctor(rab, 0, 4);
  const evil = {
    valueOf: () => {
      rab.resize(2 * ctor.BYTES_PER_ELEMENT);
      return 3;
    }
  };
  assert.throws(TypeError, () => {
    TypedArrayFillHelper(fixedLength, evil, 1, 2);
  });
}
for (let ctor of ctors) {
  const rab = CreateResizableArrayBuffer(4 * ctor.BYTES_PER_ELEMENT, 8 * ctor.BYTES_PER_ELEMENT);
  const fixedLength = new ctor(rab, 0, 4);
  const evil = {
    valueOf: () => {
      rab.resize(2 * ctor.BYTES_PER_ELEMENT);
      return 1;
    }
  };
  assert.throws(TypeError, () => {
    TypedArrayFillHelper(fixedLength, 3, evil, 2);
  });
}
for (let ctor of ctors) {
  const rab = CreateResizableArrayBuffer(4 * ctor.BYTES_PER_ELEMENT, 8 * ctor.BYTES_PER_ELEMENT);
  const fixedLength = new ctor(rab, 0, 4);
  const evil = {
    valueOf: () => {
      rab.resize(2 * ctor.BYTES_PER_ELEMENT);
      return 2;
    }
  };
  assert.throws(TypeError, () => {
    TypedArrayFillHelper(fixedLength, 3, 1, evil);
  });
}
