// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 19.1.2.18
description: Object.setPrototypeOf invoked with a non-object-coercible value
info: |
    1. Let O be RequireObjectCoercible(O).
    2. ReturnIfAbrupt(O).
---*/

assert.throws(TypeError, function() {
  Object.setPrototypeOf(null);
});

assert.throws(TypeError, function() {
  Object.setPrototypeOf(undefined);
});
