// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    When the Object function is called with one argument value,
    and the value neither is null nor undefined, and is supplied, return ToObject(value)
es5id: 15.2.1.1_A2_T13
description: Calling Object function with boolean expression as argument value
---*/

var obj = Object((1 === 1) && (!false));

assert.sameValue(obj.constructor, Boolean, 'The value of obj.constructor is expected to equal the value of Boolean');
assert.sameValue(typeof obj, "object", 'The value of `typeof obj` is expected to be "object"');
assert(!!obj, 'The value of !!obj is expected to be true');
assert.notSameValue(obj, true, 'The value of obj is not true');
