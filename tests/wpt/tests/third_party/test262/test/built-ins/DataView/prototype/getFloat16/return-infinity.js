// Copyright (C) 2024 Kevin Gibbons. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-dataview.prototype.getfloat16
description: >
  Return Infinity values
features: [Float16Array, DataView.prototype.setUint8]
---*/

var buffer = new ArrayBuffer(4);
var sample = new DataView(buffer, 0);

sample.setUint8(0, 124); // 0b01111100
sample.setUint8(1, 0);
sample.setUint8(2, 252); // 0b11111100
sample.setUint8(3, 0);

assert.sameValue(sample.getFloat16(0), Infinity);
assert.sameValue(sample.getFloat16(2), -Infinity);
