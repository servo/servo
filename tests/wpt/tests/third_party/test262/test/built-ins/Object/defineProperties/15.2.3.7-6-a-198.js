// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.7-6-a-198
description: >
    Object.defineProperties - 'O' is an Array, 'P' is an array index
    named property, 'P' property doesn't exist in 'O', test
    [[Enumerable]] of 'P' property in 'Attributes' is set as false
    value if [[Enumerable]] is absent in data descriptor 'desc'
    (15.4.5.1 step 4.c)
---*/

var arr = [];
var isOwnProperty = false;
var canEnumerable = false;

Object.defineProperties(arr, {
  "0": {
    value: 1001,
    writable: true,
    configurable: true
  }
});

isOwnProperty = arr.hasOwnProperty("0");
for (var i in arr) {
  if (i === "0") {
    canEnumerable = true;
  }
}

assert(isOwnProperty, 'isOwnProperty !== true');
assert.sameValue(canEnumerable, false, 'canEnumerable');
assert.sameValue(arr[0], 1001, 'arr[0]');
