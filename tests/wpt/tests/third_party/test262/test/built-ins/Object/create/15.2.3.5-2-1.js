// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    create sets the [[Prototype]] of the created object to first parameter.
    This can be checked using isPrototypeOf, or getPrototypeOf.
es5id: 15.2.3.5-2-1
description: Object.create creates new Object
---*/

function base() {}
var b = new base();
var prop = new Object();
var d = Object.create(b);

assert.sameValue(typeof d, 'object', 'typeof d');
