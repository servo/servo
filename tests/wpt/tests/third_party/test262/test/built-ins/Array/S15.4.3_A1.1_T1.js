// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    The value of the internal [[Prototype]] property of
    the Array constructor is the Function prototype object
es5id: 15.4.3_A1.1_T1
description: >
    Create new property of Function.prototype. When Array constructor
    has this property
---*/

Function.prototype.myproperty = 1;

assert.sameValue(Array.myproperty, 1, 'The value of Array.myproperty is expected to be 1');
assert.sameValue(Array.hasOwnProperty('myproperty'), false, 'Array.hasOwnProperty("myproperty") must return false');
