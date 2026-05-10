// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    When the Object function is called with one argument value,
    and the value neither is null nor undefined, and is supplied, return ToObject(value)
es5id: 15.2.1.1_A2_T10
description: Calling Object function with array of numbers as argument value
---*/

var arr = [1, 2, 3];

assert.sameValue(typeof arr, 'object', 'The value of `typeof arr` is expected to be "object"');

var n_obj = Object(arr);

arr.push(4);

assert.sameValue(n_obj, arr, 'The value of n_obj is expected to equal the value of arr');
assert.sameValue(n_obj[3], 4, 'The value of n_obj[3] is expected to be 4');
