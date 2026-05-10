// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 25.1.3.27
description: Error thrown coercing `this` value to a string
info: |
    1. Let O be RequireObjectCoercible(this value).
    2. Let S be ToString(O).
    3. ReturnIfAbrupt(S).
features: [Symbol.iterator]
---*/

var obj = {
  toString: function() {
    throw new Test262Error();
  }
};

assert.throws(Test262Error, function() {
  String.prototype[Symbol.iterator].call(obj);
});
