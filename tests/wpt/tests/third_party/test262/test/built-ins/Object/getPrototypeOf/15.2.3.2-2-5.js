// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.2-2-5
description: >
    Object.getPrototypeOf returns the [[Prototype]] of its parameter
    (Array)
---*/

assert.sameValue(Object.getPrototypeOf(Array), Function.prototype, 'Object.getPrototypeOf(Array)');
