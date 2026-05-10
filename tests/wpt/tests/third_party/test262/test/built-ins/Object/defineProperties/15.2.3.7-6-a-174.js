// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.7-6-a-174
description: >
    Object.defineProperties - 'O' is an Array, 'P' is the length
    property of 'O', the [[Value]] field of 'desc' is less than value
    of  the length property, test the configurable large index named
    property of 'O' can be deleted (15.4.5.1 step 3.l.ii)
---*/

var arr = [0, 1];

Object.defineProperties(arr, {
  length: {
    value: 1
  }
});

assert.sameValue(arr.hasOwnProperty("1"), false, 'arr.hasOwnProperty("1")');
