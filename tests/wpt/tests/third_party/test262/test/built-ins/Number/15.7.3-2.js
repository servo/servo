// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.7.3-2
description: >
    Number constructor - [[Prototype]] is the Function prototype
    object (using getPrototypeOf)
---*/

var p = Object.getPrototypeOf(Number);

assert.sameValue(p, Function.prototype, 'p');
