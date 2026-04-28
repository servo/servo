// Copyright (C) 2024 Kevin Gibbons. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-dataview.prototype.setfloat16
description: >
  Throws a RangeError if getIndex < 0
features: [Float16Array]
---*/

var buffer = new ArrayBuffer(12);
var sample = new DataView(buffer, 0);

assert.throws(RangeError, function() {
  sample.setFloat16(-1, 39);
}, "-1");
assert.sameValue(sample.getFloat32(0), 0, "-1 - no value was set");

assert.throws(RangeError, function() {
  sample.setFloat16(-Infinity, 39);
}, "-Infinity");
assert.sameValue(sample.getFloat32(0), 0, "-Infinity - no value was set");
