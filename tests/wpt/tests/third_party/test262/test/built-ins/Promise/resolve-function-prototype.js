// Copyright (C) 2015 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es6id: 25.4.1.3.2
description: The [[Prototype]] of Promise Resolve functions
info: |
  17 ECMAScript Standard Built-in Objects:
    Unless otherwise specified every built-in function and every built-in
    constructor has the Function prototype object, which is the initial
    value of the expression Function.prototype (19.2.3), as the value of
    its [[Prototype]] internal slot.
---*/

var resolveFunction;
new Promise(function(resolve, reject) {
  resolveFunction = resolve;
});

assert.sameValue(Object.getPrototypeOf(resolveFunction), Function.prototype);
