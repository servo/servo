// Copyright (C) 2024 Kevin Gibbons. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-dataview.prototype.getfloat16
description: >
  Return -0
features: [Float16Array]
---*/

var buffer = new ArrayBuffer(8);
var sample = new DataView(buffer, 0);

sample.setUint8(0, 128);
sample.setUint8(1, 0);

var result = sample.getFloat16(0);
assert.sameValue(result, -0);
