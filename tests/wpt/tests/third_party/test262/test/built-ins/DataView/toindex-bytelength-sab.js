// Copyright (C) 2016 the V8 project authors. All rights reserved.
// Copyright (C) 2017 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-dataview-buffer-byteoffset-bytelength
description: >
  ToIndex conversions on byteLength
info: |
  24.2.2.1 DataView ( buffer, byteOffset, byteLength )

  ...
  8. If byteLength is either not present or undefined, then
    a. Let viewByteLength be bufferByteLength - offset.
  9. Else,
    a. Let viewByteLength be ? ToIndex(byteLength).
    b. If offset + viewByteLength > bufferByteLength, throw a RangeError
    exception.
  ...

  ToIndex( value )

  1. If value is undefined, then
    a. Let index be 0.
  2. Else,
    a. Let integerIndex be ? ToInteger(value).
    b. If integerIndex < 0, throw a RangeError exception.
    c. Let index be ! ToLength(integerIndex).
    d. If SameValueZero(integerIndex, index) is false, throw a RangeError exception.
  3. Return index.
features: [SharedArrayBuffer]
---*/

var obj1 = {
  valueOf: function() {
    return 3;
  }
};

var obj2 = {
  toString: function() {
    return 4;
  }
};

var sample;
var ab = new SharedArrayBuffer(42);

sample = new DataView(ab, 0, -0);
assert.sameValue(sample.byteLength, 0, "-0");

sample = new DataView(ab, 0, obj1);
assert.sameValue(sample.byteLength, 3, "object's valueOf");

sample = new DataView(ab, 0, obj2);
assert.sameValue(sample.byteLength, 4, "object's toString");

sample = new DataView(ab, 0, "");
assert.sameValue(sample.byteLength, 0, "the Empty string");

sample = new DataView(ab, 0, "0");
assert.sameValue(sample.byteLength, 0, "string '0'");

sample = new DataView(ab, 0, "1");
assert.sameValue(sample.byteLength, 1, "string '1'");

sample = new DataView(ab, 0, true);
assert.sameValue(sample.byteLength, 1, "true");

sample = new DataView(ab, 0, false);
assert.sameValue(sample.byteLength, 0, "false");

sample = new DataView(ab, 0, NaN);
assert.sameValue(sample.byteLength, 0, "NaN");

sample = new DataView(ab, 0, null);
assert.sameValue(sample.byteLength, 0, "null");

sample = new DataView(ab, 0, 0.1);
assert.sameValue(sample.byteLength, 0, "0.1");

sample = new DataView(ab, 0, 0.9);
assert.sameValue(sample.byteLength, 0, "0.9");

sample = new DataView(ab, 0, 1.1);
assert.sameValue(sample.byteLength, 1, "1.1");

sample = new DataView(ab, 0, 1.9);
assert.sameValue(sample.byteLength, 1, "1.9");

sample = new DataView(ab, 0, -0.1);
assert.sameValue(sample.byteLength, 0, "-0.1");

sample = new DataView(ab, 0, -0.99999);
assert.sameValue(sample.byteLength, 0, "-0.99999");
