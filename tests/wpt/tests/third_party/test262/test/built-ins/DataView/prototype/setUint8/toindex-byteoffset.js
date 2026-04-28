// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-dataview.prototype.setuint8
description: >
  ToIndex conversions on byteOffset
info: |
  24.2.4.18 DataView.prototype.setUint8 ( byteOffset, value )

  1. Let v be the this value.
  2. Return ? SetViewValue(v, byteOffset, true, "Uint8", value).

  24.2.1.2 SetViewValue ( view, requestIndex, isLittleEndian, type, value )

  ...
  4. Let getIndex be ? ToIndex(requestIndex).
  ...
features: [Uint8Array]
---*/

var buffer = new ArrayBuffer(12);
var sample = new DataView(buffer, 0);
var typedArray = new Uint8Array(buffer, 0);

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

sample.setUint8(0, 0);
sample.setUint8(-0, 42);
assert.sameValue(typedArray[0], 42, "-0");

sample.setUint8(3, 0);
sample.setUint8(obj1, 42);
assert.sameValue(typedArray[3], 42, "object's valueOf");

sample.setUint8(4, 0);
sample.setUint8(obj2, 42);
assert.sameValue(typedArray[4], 42, "object's toString");

sample.setUint8(0, 0);
sample.setUint8("", 42);
assert.sameValue(typedArray[0], 42, "the Empty string");

sample.setUint8(0, 0);
sample.setUint8("0", 42);
assert.sameValue(typedArray[0], 42, "string '0'");

sample.setUint8(2, 0);
sample.setUint8("2", 42);
assert.sameValue(typedArray[2], 42, "string '2'");

sample.setUint8(1, 0);
sample.setUint8(true, 42);
assert.sameValue(typedArray[1], 42, "true");

sample.setUint8(0, 0);
sample.setUint8(false, 42);
assert.sameValue(typedArray[0], 42, "false");

sample.setUint8(0, 0);
sample.setUint8(NaN, 42);
assert.sameValue(typedArray[0], 42, "NaN");

sample.setUint8(0, 0);
sample.setUint8(null, 42);
assert.sameValue(typedArray[0], 42, "null");

sample.setUint8(0, 0);
sample.setUint8(0.1, 42);
assert.sameValue(typedArray[0], 42, "0.1");

sample.setUint8(0, 0);
sample.setUint8(0.9, 42);
assert.sameValue(typedArray[0], 42, "0.9");

sample.setUint8(1, 0);
sample.setUint8(1.1, 42);
assert.sameValue(typedArray[1], 42, "1.1");

sample.setUint8(1, 0);
sample.setUint8(1.9, 42);
assert.sameValue(typedArray[1], 42, "1.9");

sample.setUint8(0, 0);
sample.setUint8(-0.1, 42);
assert.sameValue(typedArray[0], 42, "-0.1");

sample.setUint8(0, 0);
sample.setUint8(-0.99999, 42);
assert.sameValue(typedArray[0], 42, "-0.99999");

sample.setUint8(0, 0);
sample.setUint8(undefined, 42);
assert.sameValue(typedArray[0], 42, "undefined");

sample.setUint8(0, 7);
sample.setUint8();
assert.sameValue(typedArray[0], 0, "no arg");
