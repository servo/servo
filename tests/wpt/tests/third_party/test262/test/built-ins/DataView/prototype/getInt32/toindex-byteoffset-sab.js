// Copyright (C) 2016 the V8 project authors. All rights reserved.
// Copyright (C) 2017 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-dataview.prototype.getint32
description: >
  ToIndex conversions on byteOffset
info: |
  24.2.4.9 DataView.prototype.getInt32 ( byteOffset [ , littleEndian ] )

  1. Let v be the this value.
  2. If littleEndian is not present, let littleEndian be false.
  3. Return ? GetViewValue(v, byteOffset, littleEndian, "Int32").

  24.2.1.1 GetViewValue ( view, requestIndex, isLittleEndian, type )

  ...
  4. Let getIndex be ? ToIndex(requestIndex).
  ...
features: [SharedArrayBuffer]
---*/

var buffer = new SharedArrayBuffer(8);
var sample = new DataView(buffer, 0);

sample.setUint8(0, 127);
sample.setUint8(1, 255);
sample.setUint8(2, 255);
sample.setUint8(3, 255);
sample.setUint8(4, 128);
sample.setUint8(5, 255);
sample.setUint8(6, 255);
sample.setUint8(7, 255);

var obj1 = {
  valueOf: function() {
    return 2;
  }
};

var obj2 = {
  toString: function() {
    return 3;
  }
};

assert.sameValue(sample.getInt32(-0), 2147483647, "-0");
assert.sameValue(sample.getInt32(obj1), -32513, "object's valueOf");
assert.sameValue(sample.getInt32(obj2), -8323073, "object's toString");
assert.sameValue(sample.getInt32(""), 2147483647, "the Empty string");
assert.sameValue(sample.getInt32("0"), 2147483647, "string '0'");
assert.sameValue(sample.getInt32("2"), -32513, "string '2'");
assert.sameValue(sample.getInt32(true), -128, "true");
assert.sameValue(sample.getInt32(false), 2147483647, "false");
assert.sameValue(sample.getInt32(NaN), 2147483647, "NaN");
assert.sameValue(sample.getInt32(null), 2147483647, "null");
assert.sameValue(sample.getInt32(0.1), 2147483647, "0.1");
assert.sameValue(sample.getInt32(0.9), 2147483647, "0.9");
assert.sameValue(sample.getInt32(1.1), -128, "1.1");
assert.sameValue(sample.getInt32(1.9), -128, "1.9");
assert.sameValue(sample.getInt32(-0.1), 2147483647, "-0.1");
assert.sameValue(sample.getInt32(-0.99999), 2147483647, "-0.99999");
assert.sameValue(sample.getInt32(undefined), 2147483647, "undefined");
assert.sameValue(sample.getInt32(), 2147483647, "no arg");
