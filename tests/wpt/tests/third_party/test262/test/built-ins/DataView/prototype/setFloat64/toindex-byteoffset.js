// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-dataview.prototype.setfloat64
description: >
  ToIndex conversions on byteOffset
info: |
  24.2.4.14 DataView.prototype.setFloat64 ( byteOffset, value [ , littleEndian ] )

  1. Let v be the this value.
  2. If littleEndian is not present, let littleEndian be false.
  3. Return ? SetViewValue(v, byteOffset, littleEndian, "Float64", value).

  24.2.1.2 SetViewValue ( view, requestIndex, isLittleEndian, type, value )

  ...
  4. Let getIndex be ? ToIndex(requestIndex).
  ...
features: [DataView.prototype.getFloat64]
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

sample.setFloat64(0, 0);
sample.setFloat64(-0, 42);
assert.sameValue(sample.getFloat64(0), 42, "-0");

sample.setFloat64(3, 0);
sample.setFloat64(obj1, 42);
assert.sameValue(sample.getFloat64(3), 42, "object's valueOf");

sample.setFloat64(4, 0);
sample.setFloat64(obj2, 42);
assert.sameValue(sample.getFloat64(4), 42, "object's toString");

sample.setFloat64(0, 0);
sample.setFloat64("", 42);
assert.sameValue(sample.getFloat64(0), 42, "the Empty string");

sample.setFloat64(0, 0);
sample.setFloat64("0", 42);
assert.sameValue(sample.getFloat64(0), 42, "string '0'");

sample.setFloat64(2, 0);
sample.setFloat64("2", 42);
assert.sameValue(sample.getFloat64(2), 42, "string '2'");

sample.setFloat64(1, 0);
sample.setFloat64(true, 42);
assert.sameValue(sample.getFloat64(1), 42, "true");

sample.setFloat64(0, 0);
sample.setFloat64(false, 42);
assert.sameValue(sample.getFloat64(0), 42, "false");

sample.setFloat64(0, 0);
sample.setFloat64(NaN, 42);
assert.sameValue(sample.getFloat64(0), 42, "NaN");

sample.setFloat64(0, 0);
sample.setFloat64(null, 42);
assert.sameValue(sample.getFloat64(0), 42, "null");

sample.setFloat64(0, 0);
sample.setFloat64(0.1, 42);
assert.sameValue(sample.getFloat64(0), 42, "0.1");

sample.setFloat64(0, 0);
sample.setFloat64(0.9, 42);
assert.sameValue(sample.getFloat64(0), 42, "0.9");

sample.setFloat64(1, 0);
sample.setFloat64(1.1, 42);
assert.sameValue(sample.getFloat64(1), 42, "1.1");

sample.setFloat64(1, 0);
sample.setFloat64(1.9, 42);
assert.sameValue(sample.getFloat64(1), 42, "1.9");

sample.setFloat64(0, 0);
sample.setFloat64(-0.1, 42);
assert.sameValue(sample.getFloat64(0), 42, "-0.1");

sample.setFloat64(0, 0);
sample.setFloat64(-0.99999, 42);
assert.sameValue(sample.getFloat64(0), 42, "-0.99999");

sample.setFloat64(0, 0);
sample.setFloat64(undefined, 42);
assert.sameValue(sample.getFloat64(0), 42, "undefined");

sample.setFloat64(0, 7);
sample.setFloat64();
assert.sameValue(sample.getFloat64(0), NaN, "no arg");
