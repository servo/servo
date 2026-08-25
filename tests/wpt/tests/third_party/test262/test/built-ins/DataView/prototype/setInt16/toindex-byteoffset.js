// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-dataview.prototype.setint16
description: >
  ToIndex conversions on byteOffset
info: |
  24.2.4.16 DataView.prototype.setInt16 ( byteOffset, value [ , littleEndian ] )

  1. Let v be the this value.
  2. If littleEndian is not present, let littleEndian be false.
  3. Return ? SetViewValue(v, byteOffset, littleEndian, "Int16", value).

  24.2.1.2 SetViewValue ( view, requestIndex, isLittleEndian, type, value )

  ...
  4. Let getIndex be ? ToIndex(requestIndex).
  ...
features: [DataView.prototype.getInt16]
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

sample.setInt16(0, 0);
sample.setInt16(-0, 42);
assert.sameValue(sample.getInt16(0), 42, "-0");

sample.setInt16(3, 0);
sample.setInt16(obj1, 42);
assert.sameValue(sample.getInt16(3), 42, "object's valueOf");

sample.setInt16(4, 0);
sample.setInt16(obj2, 42);
assert.sameValue(sample.getInt16(4), 42, "object's toString");

sample.setInt16(0, 0);
sample.setInt16("", 42);
assert.sameValue(sample.getInt16(0), 42, "the Empty string");

sample.setInt16(0, 0);
sample.setInt16("0", 42);
assert.sameValue(sample.getInt16(0), 42, "string '0'");

sample.setInt16(2, 0);
sample.setInt16("2", 42);
assert.sameValue(sample.getInt16(2), 42, "string '2'");

sample.setInt16(1, 0);
sample.setInt16(true, 42);
assert.sameValue(sample.getInt16(1), 42, "true");

sample.setInt16(0, 0);
sample.setInt16(false, 42);
assert.sameValue(sample.getInt16(0), 42, "false");

sample.setInt16(0, 0);
sample.setInt16(NaN, 42);
assert.sameValue(sample.getInt16(0), 42, "NaN");

sample.setInt16(0, 0);
sample.setInt16(null, 42);
assert.sameValue(sample.getInt16(0), 42, "null");

sample.setInt16(0, 0);
sample.setInt16(0.1, 42);
assert.sameValue(sample.getInt16(0), 42, "0.1");

sample.setInt16(0, 0);
sample.setInt16(0.9, 42);
assert.sameValue(sample.getInt16(0), 42, "0.9");

sample.setInt16(1, 0);
sample.setInt16(1.1, 42);
assert.sameValue(sample.getInt16(1), 42, "1.1");

sample.setInt16(1, 0);
sample.setInt16(1.9, 42);
assert.sameValue(sample.getInt16(1), 42, "1.9");

sample.setInt16(0, 0);
sample.setInt16(-0.1, 42);
assert.sameValue(sample.getInt16(0), 42, "-0.1");

sample.setInt16(0, 0);
sample.setInt16(-0.99999, 42);
assert.sameValue(sample.getInt16(0), 42, "-0.99999");

sample.setInt16(0, 0);
sample.setInt16(undefined, 42);
assert.sameValue(sample.getInt16(0), 42, "undefined");

sample.setInt16(0, 7);
sample.setInt16();
assert.sameValue(sample.getInt16(0), 0, "no arg");
