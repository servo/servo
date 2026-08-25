// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.7-6-a-265
description: >
    Object.defineProperties - 'O' is an Array, 'P' is an array index
    named property, test the length property of 'O' is set as
    ToUint32('P') + 1 if ToUint32('P') is greater than value of the
    length property in 'O' (15.4.5.1 step 4.e.ii)
---*/

var arr = [];

Object.defineProperties(arr, {
  "5": {
    value: 26
  }
});

assert.sameValue(arr.length, 6, 'arr.length');
assert.sameValue(arr[5], 26, 'arr[5]');
