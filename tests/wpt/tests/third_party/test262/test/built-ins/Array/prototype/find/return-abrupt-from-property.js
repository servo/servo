// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-array.prototype.find
description: >
  Returns abrupt from getting property value from `this`.
info: |
  22.1.3.8 Array.prototype.find ( predicate[ , thisArg ] )

  ...
  7. Let k be 0.
  8. Repeat, while k < len
    a. Let Pk be ToString(k).
    b. Let kValue be Get(O, Pk).
    c. ReturnIfAbrupt(kValue).
  ...
---*/

var o = {
  length: 1
};

Object.defineProperty(o, 0, {
  get: function() {
    throw new Test262Error();
  }
});

assert.throws(Test262Error, function() {
  [].find.call(o, function() {});
});
