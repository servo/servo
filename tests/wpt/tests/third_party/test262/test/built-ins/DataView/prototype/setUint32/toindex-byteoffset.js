// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-dataview.prototype.setuint32
description: >
  ToIndex conversions on byteOffset
info: |
  24.2.4.20 DataView.prototype.setUint32 ( byteOffset, value [ , littleEndian ] )

  1. Let v be the this value.
  2. If littleEndian is not present, let littleEndian be false.
  3. Return ? SetViewValue(v, byteOffset, littleEndian, "Uint32", value).

  24.2.1.2 SetViewValue ( view, requestIndex, isLittleEndian, type, value )

  ...
  4. Let getIndex be ? ToIndex(requestIndex).
  ...
features: [DataView.prototype.getUint32]
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

sample.setUint32(0, 0);
sample.setUint32(-0, 42);
assert.sameValue(sample.getUint32(0), 42, "-0");

sample.setUint32(3, 0);
sample.setUint32(obj1, 42);
assert.sameValue(sample.getUint32(3), 42, "object's valueOf");

sample.setUint32(4, 0);
sample.setUint32(obj2, 42);
assert.sameValue(sample.getUint32(4), 42, "object's toString");

sample.setUint32(0, 0);
sample.setUint32("", 42);
assert.sameValue(sample.getUint32(0), 42, "the Empty string");

sample.setUint32(0, 0);
sample.setUint32("0", 42);
assert.sameValue(sample.getUint32(0), 42, "string '0'");

sample.setUint32(2, 0);
sample.setUint32("2", 42);
assert.sameValue(sample.getUint32(2), 42, "string '2'");

sample.setUint32(1, 0);
sample.setUint32(true, 42);
assert.sameValue(sample.getUint32(1), 42, "true");

sample.setUint32(0, 0);
sample.setUint32(false, 42);
assert.sameValue(sample.getUint32(0), 42, "false");

sample.setUint32(0, 0);
sample.setUint32(NaN, 42);
assert.sameValue(sample.getUint32(0), 42, "NaN");

sample.setUint32(0, 0);
sample.setUint32(null, 42);
assert.sameValue(sample.getUint32(0), 42, "null");

sample.setUint32(0, 0);
sample.setUint32(0.1, 42);
assert.sameValue(sample.getUint32(0), 42, "0.1");

sample.setUint32(0, 0);
sample.setUint32(0.9, 42);
assert.sameValue(sample.getUint32(0), 42, "0.9");

sample.setUint32(1, 0);
sample.setUint32(1.1, 42);
assert.sameValue(sample.getUint32(1), 42, "1.1");

sample.setUint32(1, 0);
sample.setUint32(1.9, 42);
assert.sameValue(sample.getUint32(1), 42, "1.9");

sample.setUint32(0, 0);
sample.setUint32(-0.1, 42);
assert.sameValue(sample.getUint32(0), 42, "-0.1");

sample.setUint32(0, 0);
sample.setUint32(-0.99999, 42);
assert.sameValue(sample.getUint32(0), 42, "-0.99999");

sample.setUint32(0, 0);
sample.setUint32(undefined, 42);
assert.sameValue(sample.getUint32(0), 42, "undefined");

sample.setUint32(0, 7);
sample.setUint32();
assert.sameValue(sample.getUint32(0), 0, "no arg");
