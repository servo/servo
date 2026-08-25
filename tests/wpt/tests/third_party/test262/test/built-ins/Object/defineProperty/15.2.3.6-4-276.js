// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-4-276
description: >
    Object.defineProperty - 'O' is an Array, 'name' is an array index
    named property, test the length property of 'O' is set as
    ToUint32('name') + 1 if ToUint32('name') is greater than value of
    the length property in 'O' (15.4.5.1 step 4.e.ii)
---*/

var arrObj = [];

Object.defineProperty(arrObj, "5", {
  value: 3
});

assert.sameValue(arrObj.length, 6, 'arrObj.length');
assert.sameValue(arrObj[5], 3, 'arrObj[5]');
