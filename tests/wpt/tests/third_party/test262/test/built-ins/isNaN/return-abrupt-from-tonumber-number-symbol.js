// Copyright (C) 2016 The V8 Project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-isnan-number
description: >
  Throws a TypeError if number is a Symbol
info: |
  isNaN (number)

  1. Let num be ? ToNumber(number).
features: [Symbol]
---*/

var s = Symbol(1);

assert.throws(TypeError, function() {
  isNaN(s);
});
