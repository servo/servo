// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.2-2-20
description: >
    Object.getPrototypeOf returns the [[Prototype]] of its parameter
    (Function Object)
---*/

var obj = function(a, b) {
  return a + b;
};

assert.sameValue(Object.getPrototypeOf(obj), Function.prototype, 'Object.getPrototypeOf(obj)');
