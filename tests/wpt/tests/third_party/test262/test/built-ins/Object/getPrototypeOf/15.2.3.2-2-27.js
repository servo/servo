// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.2-2-27
description: >
    Object.getPrototypeOf returns the [[Prototype]] of its parameter
    (Error object)
---*/

var obj = new Error();

assert.sameValue(Object.getPrototypeOf(obj), Error.prototype, 'Object.getPrototypeOf(obj)');
