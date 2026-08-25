// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.7-6-a-165
description: >
    Object.defineProperties - 'O' is an Array, 'P' is the length
    property of 'O', the [[Value]] field of 'desc' is less than value
    of  the length property, test the length property is decreased by
    1 (15.4.5.1 step 3.l.i)
---*/

var arr = [0, 1, 2];

Object.defineProperty(arr, "1", {
  configurable: false
});

Object.defineProperty(arr, "2", {
  configurable: true
});
assert.throws(TypeError, function() {
  Object.defineProperties(arr, {
    length: {
      value: 1
    }
  });
});
assert.sameValue(arr.length, 2, 'arr.length');
assert.sameValue(arr.hasOwnProperty("2"), false, 'arr.hasOwnProperty("2")');
assert.sameValue(arr[0], 0, 'arr[0]');
assert.sameValue(arr[1], 1, 'arr[1]');
