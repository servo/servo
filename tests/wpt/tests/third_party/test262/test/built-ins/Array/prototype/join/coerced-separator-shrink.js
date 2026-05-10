// Copyright 2023 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.join
description: >
  Array.p.join behaves correctly when the receiver is shrunk during
  argument coercion
includes: [resizableArrayBufferUtils.js]
features: [resizable-arraybuffer]
---*/

// Shrinking + fixed-length TA.
for (let ctor of ctors) {
  const rab = CreateResizableArrayBuffer(4 * ctor.BYTES_PER_ELEMENT, 8 * ctor.BYTES_PER_ELEMENT);
  const fixedLength = new ctor(rab, 0, 4);
  let evil = {
    toString: () => {
      rab.resize(2 * ctor.BYTES_PER_ELEMENT);
      return '.';
    }
  };
  // We iterate 4 elements, since it was the starting length, but the TA is
  // OOB right after parameter conversion, so all elements are converted to
  // the empty string.
  assert.sameValue(Array.prototype.join.call(fixedLength, evil), '...');
}

// Shrinking + length-tracking TA.
for (let ctor of ctors) {
  const rab = CreateResizableArrayBuffer(4 * ctor.BYTES_PER_ELEMENT, 8 * ctor.BYTES_PER_ELEMENT);
  const lengthTracking = new ctor(rab);
  let evil = {
    toString: () => {
      rab.resize(2 * ctor.BYTES_PER_ELEMENT);
      return '.';
    }
  };
  // We iterate 4 elements, since it was the starting length. Elements beyond
  // the new length are converted to the empty string.
  assert.sameValue(Array.prototype.join.call(lengthTracking, evil), '0.0..');
}
