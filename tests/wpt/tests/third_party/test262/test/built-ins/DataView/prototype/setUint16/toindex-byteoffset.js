// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-dataview.prototype.setuint16
description: >
  ToIndex conversions on byteOffset
info: |
  24.2.4.19 DataView.prototype.setUint16 ( byteOffset, value [ , littleEndian ] )

  1. Let v be the this value.
  2. If littleEndian is not present, let littleEndian be false.
  3. Return ? SetViewValue(v, byteOffset, littleEndian, "Uint16", value).

  24.2.1.2 SetViewValue ( view, requestIndex, isLittleEndian, type, value )

  ...
  4. Let getIndex be ? ToIndex(requestIndex).
  ...
features: [DataView.prototype.getUint16]
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

sample.setUint16(0, 0);
sample.setUint16(-0, 42);
assert.sameValue(sample.getUint16(0), 42, "-0");

sample.setUint16(3, 0);
sample.setUint16(obj1, 42);
assert.sameValue(sample.getUint16(3), 42, "object's valueOf");

sample.setUint16(4, 0);
sample.setUint16(obj2, 42);
assert.sameValue(sample.getUint16(4), 42, "object's toString");

sample.setUint16(0, 0);
sample.setUint16("", 42);
assert.sameValue(sample.getUint16(0), 42, "the Empty string");

sample.setUint16(0, 0);
sample.setUint16("0", 42);
assert.sameValue(sample.getUint16(0), 42, "string '0'");

sample.setUint16(2, 0);
sample.setUint16("2", 42);
assert.sameValue(sample.getUint16(2), 42, "string '2'");

sample.setUint16(1, 0);
sample.setUint16(true, 42);
assert.sameValue(sample.getUint16(1), 42, "true");

sample.setUint16(0, 0);
sample.setUint16(false, 42);
assert.sameValue(sample.getUint16(0), 42, "false");

sample.setUint16(0, 0);
sample.setUint16(NaN, 42);
assert.sameValue(sample.getUint16(0), 42, "NaN");

sample.setUint16(0, 0);
sample.setUint16(null, 42);
assert.sameValue(sample.getUint16(0), 42, "null");

sample.setUint16(0, 0);
sample.setUint16(0.1, 42);
assert.sameValue(sample.getUint16(0), 42, "0.1");

sample.setUint16(0, 0);
sample.setUint16(0.9, 42);
assert.sameValue(sample.getUint16(0), 42, "0.9");

sample.setUint16(1, 0);
sample.setUint16(1.1, 42);
assert.sameValue(sample.getUint16(1), 42, "1.1");

sample.setUint16(1, 0);
sample.setUint16(1.9, 42);
assert.sameValue(sample.getUint16(1), 42, "1.9");

sample.setUint16(0, 0);
sample.setUint16(-0.1, 42);
assert.sameValue(sample.getUint16(0), 42, "-0.1");

sample.setUint16(0, 0);
sample.setUint16(-0.99999, 42);
assert.sameValue(sample.getUint16(0), 42, "-0.99999");

sample.setUint16(0, 0);
sample.setUint16(undefined, 42);
assert.sameValue(sample.getUint16(0), 42, "undefined");

sample.setUint16(0, 7);
sample.setUint16();
assert.sameValue(sample.getUint16(0), 0, "no arg");
