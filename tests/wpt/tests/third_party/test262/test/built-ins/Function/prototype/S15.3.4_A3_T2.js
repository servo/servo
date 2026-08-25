// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    The value of the internal [[Prototype]] property of the Function
    prototype object is the Object prototype object (15.3.2.1)
es5id: 15.3.4_A3_T2
description: >
    Add new property to Object.prototype and check it at
    Function.prototype
---*/

Object.prototype.indicator = 1;

assert.sameValue(Function.prototype.indicator, 1, 'The value of Function.prototype.indicator is expected to be 1');
