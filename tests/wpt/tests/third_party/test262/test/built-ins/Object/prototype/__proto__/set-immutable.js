// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-object.prototype.__proto__
description: Called on an immutable prototype exotic object
info: |
    [...]
    4. Let status be ? O.[[SetPrototypeOf]](proto).
    5. If status is false, throw a TypeError exception.
features: [__proto__]
---*/

Object.prototype.__proto__ = null;

assert.throws(TypeError, function() {
  Object.prototype.__proto__ = {};
});

assert.sameValue(Object.getPrototypeOf(Object.prototype), null);
