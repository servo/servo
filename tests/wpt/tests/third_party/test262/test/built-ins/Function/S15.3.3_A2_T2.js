// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    The value of the internal [[Prototype]] property of the Function constructor
    is the Function prototype object
es5id: 15.3.3_A2_T2
description: Add new property to Function.prototype and check it
---*/

Function.prototype.indicator = 1;

assert.sameValue(Function.indicator, 1, 'The value of Function.indicator is expected to be 1');
