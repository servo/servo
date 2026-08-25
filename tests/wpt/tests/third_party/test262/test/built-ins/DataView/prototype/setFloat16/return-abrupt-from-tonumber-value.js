// Copyright (C) 2024 Kevin Gibbons. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-dataview.prototype.setfloat16
description: >
  Return abrupt from ToNumber(value)
features: [Float16Array]
---*/

var buffer = new ArrayBuffer(4);
var sample = new DataView(buffer, 0);

var bo1 = {
  valueOf: function() {
    throw new Test262Error();
  }
};

var bo2 = {
  toString: function() {
    throw new Test262Error();
  }
};

assert.throws(Test262Error, function() {
  sample.setFloat16(0, bo1);
}, "valueOf");

assert.throws(Test262Error, function() {
  sample.setFloat16(0, bo2);
}, "toString");
