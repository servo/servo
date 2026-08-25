// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 25.1.3.27
description: The `this` value cannot be coerced into an object
info: |
    1. Let O be RequireObjectCoercible(this value).
    2. Let S be ToString(O).
    3. ReturnIfAbrupt(S).
features: [Symbol.iterator]
---*/

assert.throws(TypeError, function() {
  String.prototype[Symbol.iterator].call(undefined);
});

assert.throws(TypeError, function() {
  String.prototype[Symbol.iterator].call(null);
});
