// Copyright 2023 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-%typedarray%.prototype.join
description: >
  TypedArray.p.join behaves correctly when the receiver is grown during
  argument coercion
includes: [resizableArrayBufferUtils.js]
features: [resizable-arraybuffer]
---*/

// Growing + fixed-length TA.
for (let ctor of ctors) {
  const rab = CreateResizableArrayBuffer(4 * ctor.BYTES_PER_ELEMENT, 8 * ctor.BYTES_PER_ELEMENT);
  const fixedLength = new ctor(rab, 0, 4);
  let evil = {
    toString: () => {
      rab.resize(6 * ctor.BYTES_PER_ELEMENT);
      return '.';
    }
  };
  assert.sameValue(fixedLength.join(evil), '0.0.0.0');
}

// Growing + length-tracking TA.
for (let ctor of ctors) {
  const rab = CreateResizableArrayBuffer(4 * ctor.BYTES_PER_ELEMENT, 8 * ctor.BYTES_PER_ELEMENT);
  const lengthTracking = new ctor(rab);
  let evil = {
    toString: () => {
      rab.resize(6 * ctor.BYTES_PER_ELEMENT);
      return '.';
    }
  };
  // We iterate 4 elements, since it was the starting length.
  assert.sameValue(lengthTracking.join(evil), '0.0.0.0');
}
