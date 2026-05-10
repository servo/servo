// Copyright (C) 2024 Kevin Gibbons. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-dataview.prototype.getfloat16
description: >
  Boolean littleEndian argument coerced in ToBoolean
features: [Float16Array, DataView.prototype.setUint8, Symbol]
---*/

var buffer = new ArrayBuffer(2);
var sample = new DataView(buffer, 0);

sample.setUint8(0, 75);
sample.setUint8(1, 76);

// False
assert.sameValue(sample.getFloat16(0), 14.59375, "no arg");
assert.sameValue(sample.getFloat16(0, undefined), 14.59375, "undefined");
assert.sameValue(sample.getFloat16(0, null), 14.59375, "null");
assert.sameValue(sample.getFloat16(0, 0), 14.59375, "0");
assert.sameValue(sample.getFloat16(0, ""), 14.59375, "the empty string");

// True
assert.sameValue(sample.getFloat16(0, {}), 17.171875, "{}");
assert.sameValue(sample.getFloat16(0, Symbol("1")), 17.171875, "symbol");
assert.sameValue(sample.getFloat16(0, 1), 17.171875, "1");
assert.sameValue(sample.getFloat16(0, "string"), 17.171875, "string");
