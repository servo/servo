// Copyright 2023 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-%typedarray%.prototype.at
description: >
  TypedArray.p.at behaves correctly on TypedArrays backed by resizable buffers
  when the TypedArray is resized during parameter conversion
includes: [resizableArrayBufferUtils.js]
features: [resizable-arraybuffer]
---*/

function TypedArrayAtHelper(ta, index) {
  const result = ta.at(index);
  return Convert(result);
}

for (let ctor of ctors) {
  const rab = CreateResizableArrayBuffer(4 * ctor.BYTES_PER_ELEMENT, 8 * ctor.BYTES_PER_ELEMENT);
  const fixedLength = new ctor(rab, 0, 4);
  let evil = {
    valueOf: () => {
      rab.resize(2);
      return 0;
    }
  };
  assert.sameValue(TypedArrayAtHelper(fixedLength, evil), undefined);
}

for (let ctor of ctors) {
  const rab = CreateResizableArrayBuffer(4 * ctor.BYTES_PER_ELEMENT, 8 * ctor.BYTES_PER_ELEMENT);
  const lengthTracking = new ctor(rab);
  let evil = {
    valueOf: () => {
      rab.resize(2);
      return -1;
    }
  };
  // The TypedArray is *not* out of bounds since it's length-tracking.
  assert.sameValue(TypedArrayAtHelper(lengthTracking, evil), undefined);
}
