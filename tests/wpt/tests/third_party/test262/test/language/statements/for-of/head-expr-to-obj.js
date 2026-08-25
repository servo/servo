// Copyright (C) 2013 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 13.6.4.12 S8.b
description: >
    The value of the expression in a for-of statement's head is subject to the
    semantics of the ToObject abstract operation.
---*/
var x;

assert.throws(TypeError, function() {
  for (x of null) {}
});

assert.throws(TypeError, function() {
  for (x of undefined) {}
});
