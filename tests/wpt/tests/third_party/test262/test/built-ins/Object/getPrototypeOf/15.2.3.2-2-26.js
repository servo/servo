// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.2-2-26
description: >
    Object.getPrototypeOf returns the [[Prototype]] of its parameter
    (RegExp object)
---*/

var obj = new RegExp();

assert.sameValue(Object.getPrototypeOf(obj), RegExp.prototype, 'Object.getPrototypeOf(obj)');
