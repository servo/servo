// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-dataview.prototype.setint8
description: >
  ToIndex conversions on byteOffset
info: |
  24.2.4.15 DataView.prototype.setInt8 ( byteOffset, value )

  1. Let v be the this value.
  2. Return ? SetViewValue(v, byteOffset, true, "Int8", value).

  24.2.1.2 SetViewValue ( view, requestIndex, isLittleEndian, type, value )

  ...
  4. Let getIndex be ? ToIndex(requestIndex).
  ...
features: [DataView.prototype.getInt8]
---*/

var buffer = new ArrayBuffer(12);
var sample = new DataView(buffer, 0);

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

sample.setInt8(0, 0);
sample.setInt8(-0, 42);
assert.sameValue(sample.getInt8(0), 42, "-0");

sample.setInt8(3, 0);
sample.setInt8(obj1, 42);
assert.sameValue(sample.getInt8(3), 42, "object's valueOf");

sample.setInt8(4, 0);
sample.setInt8(obj2, 42);
assert.sameValue(sample.getInt8(4), 42, "object's toString");

sample.setInt8(0, 0);
sample.setInt8("", 42);
assert.sameValue(sample.getInt8(0), 42, "the Empty string");

sample.setInt8(0, 0);
sample.setInt8("0", 42);
assert.sameValue(sample.getInt8(0), 42, "string '0'");

sample.setInt8(2, 0);
sample.setInt8("2", 42);
assert.sameValue(sample.getInt8(2), 42, "string '2'");

sample.setInt8(1, 0);
sample.setInt8(true, 42);
assert.sameValue(sample.getInt8(1), 42, "true");

sample.setInt8(0, 0);
sample.setInt8(false, 42);
assert.sameValue(sample.getInt8(0), 42, "false");

sample.setInt8(0, 0);
sample.setInt8(NaN, 42);
assert.sameValue(sample.getInt8(0), 42, "NaN");

sample.setInt8(0, 0);
sample.setInt8(null, 42);
assert.sameValue(sample.getInt8(0), 42, "null");

sample.setInt8(0, 0);
sample.setInt8(0.1, 42);
assert.sameValue(sample.getInt8(0), 42, "0.1");

sample.setInt8(0, 0);
sample.setInt8(0.9, 42);
assert.sameValue(sample.getInt8(0), 42, "0.9");

sample.setInt8(1, 0);
sample.setInt8(1.1, 42);
assert.sameValue(sample.getInt8(1), 42, "1.1");

sample.setInt8(1, 0);
sample.setInt8(1.9, 42);
assert.sameValue(sample.getInt8(1), 42, "1.9");

sample.setInt8(0, 0);
sample.setInt8(-0.1, 42);
assert.sameValue(sample.getInt8(0), 42, "-0.1");

sample.setInt8(0, 0);
sample.setInt8(-0.99999, 42);
assert.sameValue(sample.getInt8(0), 42, "-0.99999");

sample.setInt8(0, 0);
sample.setInt8(undefined, 42);
assert.sameValue(sample.getInt8(0), 42, "undefined");

sample.setInt8(0, 7);
sample.setInt8();
assert.sameValue(sample.getInt8(0), 0, "no arg");
