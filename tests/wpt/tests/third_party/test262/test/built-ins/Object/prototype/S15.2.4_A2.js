// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    The value of the internal [[Class]] property of Object prototype object
    is "Object"
es5id: 15.2.4_A2
description: >
    Getting the value of the internal [[Class]] property with
    Object.prototype.toString() function
---*/

var tostr = Object.prototype.toString();

assert.sameValue(tostr, "[object Object]", 'The value of tostr is expected to be "[object Object]"');
