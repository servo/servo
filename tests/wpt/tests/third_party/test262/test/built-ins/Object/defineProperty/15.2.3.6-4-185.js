// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-4-185
description: >
    Object.defineProperty - 'O' is an Array, 'name' is an array index
    named property, 'name' is boundary value 2^32 (15.4.5.1 step 4.a)
---*/

var arrObj = [];

Object.defineProperty(arrObj, 4294967296, {
  value: 100
});

assert(arrObj.hasOwnProperty("4294967296"), 'arrObj.hasOwnProperty("4294967296") !== true');
assert.sameValue(arrObj.length, 0, 'arrObj.length');
assert.sameValue(arrObj[4294967296], 100, 'arrObj[4294967296]');
