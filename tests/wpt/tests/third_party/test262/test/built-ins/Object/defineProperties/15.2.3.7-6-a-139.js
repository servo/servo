// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.7-6-a-139
description: >
    Object.defineProperties - 'O' is an Array, 'name' is the length
    property of 'O', test the [[Value]] field of 'desc' is a string
    containing an hex number (15.4.5.1 step 3.c)
---*/

var arr = [];

Object.defineProperties(arr, {
  length: {
    value: "0x00B"
  }
});

assert.sameValue(arr.length, 0x00B, 'arr.length');
