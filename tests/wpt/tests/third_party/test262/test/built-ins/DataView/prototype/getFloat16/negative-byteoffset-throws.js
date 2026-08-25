// Copyright (C) 2024 Kevin Gibbons. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-dataview.prototype.getfloat16
description: >
  Throws a RangeError if ToInteger(byteOffset) < 0
features: [Float16Array]
---*/

var buffer = new ArrayBuffer(12);
var sample = new DataView(buffer, 0);

assert.throws(RangeError, function() {
  sample.getFloat16(-1);
}, "-1");

assert.throws(RangeError, function() {
  sample.getFloat16(-Infinity);
}, "-Infinity");
