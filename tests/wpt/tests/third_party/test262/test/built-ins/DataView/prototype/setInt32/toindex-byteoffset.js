// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-dataview.prototype.setint32
description: >
  ToIndex conversions on byteOffset
info: |
  24.2.4.17 DataView.prototype.setInt32 ( byteOffset, value [ , littleEndian ] )

  1. Let v be the this value.
  2. If littleEndian is not present, let littleEndian be false.
  3. Return ? SetViewValue(v, byteOffset, littleEndian, "Int32", value).

  24.2.1.2 SetViewValue ( view, requestIndex, isLittleEndian, type, value )

  ...
  4. Let getIndex be ? ToIndex(requestIndex).
  ...
features: [DataView.prototype.getInt32]
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

sample.setInt32(0, 0);
sample.setInt32(-0, 42);
assert.sameValue(sample.getInt32(0), 42, "-0");

sample.setInt32(3, 0);
sample.setInt32(obj1, 42);
assert.sameValue(sample.getInt32(3), 42, "object's valueOf");

sample.setInt32(4, 0);
sample.setInt32(obj2, 42);
assert.sameValue(sample.getInt32(4), 42, "object's toString");

sample.setInt32(0, 0);
sample.setInt32("", 42);
assert.sameValue(sample.getInt32(0), 42, "the Empty string");

sample.setInt32(0, 0);
sample.setInt32("0", 42);
assert.sameValue(sample.getInt32(0), 42, "string '0'");

sample.setInt32(2, 0);
sample.setInt32("2", 42);
assert.sameValue(sample.getInt32(2), 42, "string '2'");

sample.setInt32(1, 0);
sample.setInt32(true, 42);
assert.sameValue(sample.getInt32(1), 42, "true");

sample.setInt32(0, 0);
sample.setInt32(false, 42);
assert.sameValue(sample.getInt32(0), 42, "false");

sample.setInt32(0, 0);
sample.setInt32(NaN, 42);
assert.sameValue(sample.getInt32(0), 42, "NaN");

sample.setInt32(0, 0);
sample.setInt32(null, 42);
assert.sameValue(sample.getInt32(0), 42, "null");

sample.setInt32(0, 0);
sample.setInt32(0.1, 42);
assert.sameValue(sample.getInt32(0), 42, "0.1");

sample.setInt32(0, 0);
sample.setInt32(0.9, 42);
assert.sameValue(sample.getInt32(0), 42, "0.9");

sample.setInt32(1, 0);
sample.setInt32(1.1, 42);
assert.sameValue(sample.getInt32(1), 42, "1.1");

sample.setInt32(1, 0);
sample.setInt32(1.9, 42);
assert.sameValue(sample.getInt32(1), 42, "1.9");

sample.setInt32(0, 0);
sample.setInt32(-0.1, 42);
assert.sameValue(sample.getInt32(0), 42, "-0.1");

sample.setInt32(0, 0);
sample.setInt32(-0.99999, 42);
assert.sameValue(sample.getInt32(0), 42, "-0.99999");

sample.setInt32(0, 0);
sample.setInt32(undefined, 42);
assert.sameValue(sample.getInt32(0), 42, "undefined");

sample.setInt32(0, 7);
sample.setInt32();
assert.sameValue(sample.getInt32(0), 0, "no arg");
