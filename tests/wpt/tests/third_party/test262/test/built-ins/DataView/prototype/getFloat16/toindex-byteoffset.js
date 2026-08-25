// Copyright (C) 2024 Kevin Gibbons. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-dataview.prototype.getfloat16
description: >
  ToIndex conversions on byteOffset
features: [Float16Array, DataView.prototype.setUint8]
---*/

var buffer = new ArrayBuffer(6);
var sample = new DataView(buffer, 0);

sample.setUint8(0, 75);
sample.setUint8(1, 76);
sample.setUint8(2, 77);
sample.setUint8(3, 78);
sample.setUint8(4, 79);
sample.setUint8(5, 80);

var obj1 = {
  valueOf: function() {
    return 3;
  }
};

var obj2 = {
  toString: function() {
    return 2;
  }
};

assert.sameValue(sample.getFloat16(-0), 14.59375, "-0");
assert.sameValue(sample.getFloat16(obj1), 25.234375, "object's valueOf");
assert.sameValue(sample.getFloat16(obj2), 21.21875, "object's toString");
assert.sameValue(sample.getFloat16(""), 14.59375, "the Empty string");
assert.sameValue(sample.getFloat16("0"), 14.59375, "string '0'");
assert.sameValue(sample.getFloat16("2"), 21.21875, "string '2'");
assert.sameValue(sample.getFloat16(true), 17.203125, "true");
assert.sameValue(sample.getFloat16(false), 14.59375, "false");
assert.sameValue(sample.getFloat16(NaN), 14.59375, "NaN");
assert.sameValue(sample.getFloat16(null), 14.59375, "null");
assert.sameValue(sample.getFloat16(0.1), 14.59375, "0.1");
assert.sameValue(sample.getFloat16(0.9), 14.59375, "0.9");
assert.sameValue(sample.getFloat16(1.1), 17.203125, "1.1");
assert.sameValue(sample.getFloat16(1.9), 17.203125, "1.9");
assert.sameValue(sample.getFloat16(-0.1), 14.59375, "-0.1");
assert.sameValue(sample.getFloat16(-0.99999), 14.59375, "-0.99999");
assert.sameValue(sample.getFloat16(undefined), 14.59375, "undefined");
assert.sameValue(sample.getFloat16(), 14.59375, "no arg");
