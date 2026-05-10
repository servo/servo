// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.2-2-24
description: >
    Object.getPrototypeOf returns the [[Prototype]] of its parameter
    (Number object)
---*/

var obj = new Number(-3);

assert.sameValue(Object.getPrototypeOf(obj), Number.prototype, 'Object.getPrototypeOf(obj)');
