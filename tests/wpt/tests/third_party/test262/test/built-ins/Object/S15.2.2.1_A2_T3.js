// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    When the Object constructor is called with one argument value and
    the value is a native ECMAScript object, do not create a new object but simply return value
es5id: 15.2.2.1_A2_T3
description: The value is an array
---*/

var arr = [1, 2, 3];

var n_obj = new Object(arr);

arr.push(4);

assert.sameValue(n_obj, arr, 'The value of n_obj is expected to equal the value of arr');
assert.sameValue(n_obj[3], 4, 'The value of n_obj[3] is expected to be 4');
