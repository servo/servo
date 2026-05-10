// Copyright (C) 2024 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-settypedarrayfromarraylike
description: >
  Value conversion shrinks and then grows the array buffer.
info: |
  23.2.3.26.2 SetTypedArrayFromArrayLike ( target, targetOffset, source )
    ...
    9. Repeat, while k < srcLength,
      a. Let Pk be ! ToString(𝔽(k)).
      b. Let value be ? Get(src, Pk).
      c. Let targetIndex be 𝔽(targetOffset + k).
      d. Perform ? TypedArraySetElement(target, targetIndex, value).
      e. Set k to k + 1.

features: [resizable-arraybuffer]
includes: [compareArray.js]
---*/

var rab = new ArrayBuffer(5, {maxByteLength: 10});
var typedArray = new Int8Array(rab);

var log = [];

var growNumber = 0

var grow = {
  valueOf() {
    log.push("grow");
    rab.resize(rab.byteLength + 1);
    return --growNumber;
  }
};

var shrinkNumber = 0

var shrink = {
  valueOf() {
    log.push("shrink");
    rab.resize(rab.byteLength - 1);
    return ++shrinkNumber;
  }
};

var array = {
  get length() {
    return 5;
  },
  0: shrink,
  1: shrink,
  2: shrink,
  3: grow,
  4: grow,
}

typedArray.set(array);

assert.compareArray(log, [
  "shrink", "shrink", "shrink", "grow", "grow",
]);

assert.compareArray(typedArray, [
  1, 2, 0, 0
]);
