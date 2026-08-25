// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.7-6-a-122
description: >
    Object.defineProperties - 'O' is an Array, 'P' is the length
    property of 'O', test setting the [[Value]] field of 'desc' to
    null actuall is set to 0 (15.4.5.1 step 3.c)
---*/

var arr = [0, 1];

Object.defineProperties(arr, {
  length: {
    value: null
  }
});

assert.sameValue(arr.length, 0, 'arr.length');
