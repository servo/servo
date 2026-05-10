// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-4-144
description: >
    Object.defineProperty - 'O' is an Array, 'name' is the length
    property of 'O', test the [[Value]] field of 'desc' is a string
    containing a number with leading zeros (15.4.5.1 step 3.c)
---*/

var arrObj = [];

Object.defineProperty(arrObj, "length", {
  value: "0002.0"
});

assert.sameValue(arrObj.length, 2, 'arrObj.length');
