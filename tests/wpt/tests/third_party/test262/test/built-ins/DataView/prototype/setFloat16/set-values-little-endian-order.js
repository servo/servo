// Copyright (C) 2024 Kevin Gibbons. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-dataview.prototype.setfloat16
description: >
  Set values with little endian order.
features: [Float16Array]
---*/

var buffer = new ArrayBuffer(2);
var sample = new DataView(buffer, 0);

var result;

result = sample.setFloat16(0, 42, true); // 01010001 01000000
assert.sameValue(result, undefined, "returns undefined #1");
assert.sameValue(sample.getFloat16(0), 2.158203125); // 01000000 01010001

result = sample.setFloat16(0, 2.158203125, true);
assert.sameValue(result, undefined, "returns undefined #2");
assert.sameValue(sample.getFloat16(0), 42);
