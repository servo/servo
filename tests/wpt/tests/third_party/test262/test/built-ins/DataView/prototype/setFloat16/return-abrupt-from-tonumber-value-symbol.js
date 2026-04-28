// Copyright (C) 2024 Kevin Gibbons. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-dataview.prototype.setfloat16
description: >
  Return abrupt from ToNumber(symbol value)
features: [Float16Array, Symbol]
---*/

var buffer = new ArrayBuffer(4);
var sample = new DataView(buffer, 0);

var s = Symbol("1");

assert.throws(TypeError, function() {
  sample.setFloat16(0, s);
});
