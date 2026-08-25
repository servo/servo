// Copyright 2023 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-object.defineproperty
description: >
  Object.defineProperty behaves correctly when the object is a
  TypedArray backed by a resizable buffer that's shrunk during argument
  coercion
includes: [compareArray.js, resizableArrayBufferUtils.js]
features: [resizable-arraybuffer]
---*/

// Fixed length.
for (let ctor of ctors) {
  const rab = CreateResizableArrayBuffer(4 * ctor.BYTES_PER_ELEMENT, 8 * ctor.BYTES_PER_ELEMENT);
  const fixedLength = new ctor(rab, 0, 4);
  const evil = {
    toString: () => {
      rab.resize(2 * ctor.BYTES_PER_ELEMENT);
      return 0;
    }
  };
  assert.throws(TypeError, () => {
    Object.defineProperty(fixedLength, evil, { value: MayNeedBigInt(fixedLength, 8) });
  });
}

// Length tracking.
for (let ctor of ctors) {
  const rab = CreateResizableArrayBuffer(4 * ctor.BYTES_PER_ELEMENT, 8 * ctor.BYTES_PER_ELEMENT);
  const lengthTracking = new ctor(rab, 0);
  const evil = {
    toString: () => {
      rab.resize(2 * ctor.BYTES_PER_ELEMENT);
      return 3;  // Index too large after resize.
    }
  };
  assert.throws(TypeError, () => {
    Object.defineProperty(lengthTracking, evil, { value: MayNeedBigInt(lengthTracking, 8) });
  });
}
