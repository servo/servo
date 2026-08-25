// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    Since calling Object as a function is identical to calling a function,
    list of arguments bracketing is allowed
es5id: 15.2.2.1_A6_T3
description: Creating an object with "new Object((null,2,3),2,3)"
---*/

var obj = new Object((null, 2, 3), 1, 2);

assert.sameValue(obj.constructor, Number, 'The value of obj.constructor is expected to equal the value of Number');
assert.sameValue(typeof obj, "object", 'The value of `typeof obj` is expected to be "object"');
assert(obj == 3, 'The result of evaluating (obj == 3) is expected to be true');
assert.notSameValue(obj, 3, 'The value of obj is not 3');
