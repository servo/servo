// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-object.prototype.__proto__
es6id: B.2.2.1
description: Cycle detection
info: |
    [...]
    4. Let status be ? O.[[SetPrototypeOf]](proto).
    5. If status is false, throw a TypeError exception.

    9.1.2.1 OrdinarySetPrototypeOf

    [...]
    6. Let p be V.
    7. Let done be false.
    8. Repeat while done is false,
       a. If p is null, let done be true.
       b. Else if SameValue(p, O) is true, return false.
       c. Else,
          i. If the [[GetPrototypeOf]] internal method of p is not the ordinary
             object internal method defined in 9.1.1, let done be true.
          ii. Else, let p be the value of p's [[Prototype]] internal slot.
features: [__proto__]
---*/

var root = {};
var intermediary = Object.create(root);
var leaf = Object.create(intermediary);

assert.throws(TypeError, function() {
  root.__proto__ = leaf;
});

assert.sameValue(Object.getPrototypeOf(root), Object.prototype);
