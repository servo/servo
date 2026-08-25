// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 19.1.2.18
description: >
    Object.setPrototypeOf invoked with a value that would create a cycle
info: |
    1. Let O be RequireObjectCoercible(O).
    2. ReturnIfAbrupt(O).
    3. If Type(proto) is neither Object nor Null, throw a TypeError exception.
    4. If Type(O) is not Object, return O.
    5. Let status be O.[[SetPrototypeOf]](proto).
    6. ReturnIfAbrupt(status).
    7. If status is false, throw a TypeError exception.
---*/

var obj = {};

assert.throws(TypeError, function() {
  Object.setPrototypeOf(Object.prototype, Array.prototype);
});
