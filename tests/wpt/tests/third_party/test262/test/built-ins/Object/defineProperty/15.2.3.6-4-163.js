// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-4-163
description: >
    Object.defineProperty - 'O' is an Array, 'name' is the length
    property of 'O', the [[Value]] field of 'desc' equals to value of
    the length property, test no TypeError is thrown when the length
    property is not writable (15.4.5.1 step 3.f.i)
---*/

var arrObj = [];

Object.defineProperty(arrObj, "length", {
  writable: false
});

Object.defineProperty(arrObj, "length", {
  value: 0
});
