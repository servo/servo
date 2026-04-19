// Copyright (C) 2024 Kevin Gibbons. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-dataview.prototype.setfloat16
description: >
  ToIndex conversions on byteOffset
features: [Float16Array]
---*/

var buffer = new ArrayBuffer(6);
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

sample.setFloat16(0, 0);
sample.setFloat16(-0, 42);
assert.sameValue(sample.getFloat16(0), 42, "-0");

sample.setFloat16(3, 0);
sample.setFloat16(obj1, 42);
assert.sameValue(sample.getFloat16(3), 42, "object's valueOf");

sample.setFloat16(4, 0);
sample.setFloat16(obj2, 42);
assert.sameValue(sample.getFloat16(4), 42, "object's toString");

sample.setFloat16(0, 0);
sample.setFloat16("", 42);
assert.sameValue(sample.getFloat16(0), 42, "the Empty string");

sample.setFloat16(0, 0);
sample.setFloat16("0", 42);
assert.sameValue(sample.getFloat16(0), 42, "string '0'");

sample.setFloat16(2, 0);
sample.setFloat16("2", 42);
assert.sameValue(sample.getFloat16(2), 42, "string '2'");

sample.setFloat16(1, 0);
sample.setFloat16(true, 42);
assert.sameValue(sample.getFloat16(1), 42, "true");

sample.setFloat16(0, 0);
sample.setFloat16(false, 42);
assert.sameValue(sample.getFloat16(0), 42, "false");

sample.setFloat16(0, 0);
sample.setFloat16(NaN, 42);
assert.sameValue(sample.getFloat16(0), 42, "NaN");

sample.setFloat16(0, 0);
sample.setFloat16(null, 42);
assert.sameValue(sample.getFloat16(0), 42, "null");

sample.setFloat16(0, 0);
sample.setFloat16(0.1, 42);
assert.sameValue(sample.getFloat16(0), 42, "0.1");

sample.setFloat16(0, 0);
sample.setFloat16(0.9, 42);
assert.sameValue(sample.getFloat16(0), 42, "0.9");

sample.setFloat16(1, 0);
sample.setFloat16(1.1, 42);
assert.sameValue(sample.getFloat16(1), 42, "1.1");

sample.setFloat16(1, 0);
sample.setFloat16(1.9, 42);
assert.sameValue(sample.getFloat16(1), 42, "1.9");

sample.setFloat16(0, 0);
sample.setFloat16(-0.1, 42);
assert.sameValue(sample.getFloat16(0), 42, "-0.1");

sample.setFloat16(0, 0);
sample.setFloat16(-0.99999, 42);
assert.sameValue(sample.getFloat16(0), 42, "-0.99999");

sample.setFloat16(0, 0);
sample.setFloat16(undefined, 42);
assert.sameValue(sample.getFloat16(0), 42, "undefined");

sample.setFloat16(0, 7);
sample.setFloat16();
assert.sameValue(sample.getFloat16(0), NaN, "no arg");
