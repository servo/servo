// Copyright (C) 2013 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 13.6.4.12 S8.b
description: >
    The value of the expression in a for-of statement's head must have an
    `@@iterator` method.
---*/
var x;

assert.throws(TypeError, function() {
  for (x of false) {}
});

assert.throws(TypeError, function() {
  for (x of 37) {}
});
