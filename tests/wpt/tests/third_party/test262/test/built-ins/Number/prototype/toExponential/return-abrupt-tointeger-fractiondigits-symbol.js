// Copyright (C) 2016 The V8 Project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-number.prototype.toexponential
description: >
  Return abrupt completion from ToInteger(symbol fractionDigits)
info: |
  Number.prototype.toExponential ( fractionDigits )

  1. Let x be ? thisNumberValue(this value).
  2. Let f be ? ToInteger(fractionDigits).
  [...]
features: [Symbol]
---*/

var fd = Symbol("1");

assert.throws(TypeError, function() {
  NaN.toExponential(fd);
});
