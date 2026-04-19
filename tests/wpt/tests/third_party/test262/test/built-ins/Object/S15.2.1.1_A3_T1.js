// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    Since calling Object as a function is identical to calling a function,
    list of arguments bracketing is allowed
es5id: 15.2.1.1_A3_T1
description: Creating an object with "Object(1,2,3)"
---*/

var obj = Object(1, 2, 3);

assert.sameValue(obj.constructor, Number, 'The value of obj.constructor is expected to equal the value of Number');
assert.sameValue(typeof obj, "object", 'The value of `typeof obj` is expected to be "object"');

assert(obj == 1, 'The result of evaluating (obj == 1) is expected to be true');
assert.notSameValue(obj, 1, 'The value of obj is not 1');

