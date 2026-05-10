// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 19.1.2.18
description: >
    Object.setPrototypeOf invoked with an object whose prototype cannot be set
info: |
    1. Let O be RequireObjectCoercible(O).
    2. ReturnIfAbrupt(O).
    3. If Type(proto) is neither Object nor Null, throw a TypeError exception.
    4. If Type(O) is not Object, return O.
    5. Let status be O.[[SetPrototypeOf]](proto).
    6. ReturnIfAbrupt(status).
features: [Proxy]
---*/

var obj = new Proxy({}, {
  setPrototypeOf: function() {
    throw new Test262Error();
  }
});

assert.throws(Test262Error, function() {
  Object.setPrototypeOf(obj, null);
});
