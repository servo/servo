// Copyright (C) 2024 Kevin Gibbons. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-dataview.prototype.setfloat16
description: >
  Index bounds checks are performed after value conversion.
features: [Float16Array]
---*/

var dataView = new DataView(new ArrayBuffer(8), 0);

var poisoned = {
  valueOf: function() {
    throw new Test262Error();
  }
};

assert.throws(Test262Error, function() {
  dataView.setFloat16(100, poisoned);
}, "setFloat16(100, poisoned)");

assert.throws(Test262Error, function() {
  dataView.setFloat16('100', poisoned);
}, "setFloat16('100', poisoned)");
