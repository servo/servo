// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.7-6-a-123
description: >
    Object.defineProperties - 'O' is an Array, 'name' is the length
    property of 'O', test the [[Value]] field of 'desc' is a boolean
    with value false (15.4.5.1 step 3.c)
---*/

var arr = [0, 1];

Object.defineProperties(arr, {
  length: {
    value: false
  }
});

assert.sameValue(arr.length, 0, 'arr.length');
