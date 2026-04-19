// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.7-6-a-203
description: >
    Object.defineProperties - 'O' is an Array, 'P' is an array index
    named property, 'P' property doesn't exist in 'O', test
    [[Enumerable]] of 'P' property in 'Attributes' is set as false
    value if [[Enumerable]] is absent in accessor descriptor 'desc'
    (15.4.5.1 step 4.c)
---*/

var arr = [];

Object.defineProperties(arr, {
  "0": {
    set: function() {},
    get: function() {},
    configurable: true
  }
});

for (var i in arr) {
  assert.sameValue(i === "0" && arr.hasOwnProperty("0"), false, 'i === "0" && arr.hasOwnProperty("0")');
}
