// Copyright (C) 2013 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-number-constructor-number-value
description: >
  Return abrupt from ToNumber(value)
info: |
  Number ( value )

  1. If no arguments were passed to this function invocation, let n be +0.
  2. Else, let n be ? ToNumber(value).
  [...]
features: [Symbol]
---*/

var s = Symbol("66");

assert.throws(TypeError, function() {
  Number(s);
}, "NewTarget is undefined");

assert.throws(TypeError, function() {
  new Number(s);
}, "NewTarget is defined");
