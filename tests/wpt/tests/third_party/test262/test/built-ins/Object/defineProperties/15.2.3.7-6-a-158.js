// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.7-6-a-158
description: >
    Object.defineProperties - 'O' is an Array, 'P' is the length
    property of 'O', the [[Value]] field of 'desc' is greater than
    value of the length property, test TypeError is thrown when the
    length property is not writable (15.4.5.1 step 3.f.i)
---*/

var arr = [];

Object.defineProperty(arr, "length", {
  writable: false
});
assert.throws(TypeError, function() {
  Object.defineProperties(arr, {
    length: {
      value: 12
    }
  });
});
assert.sameValue(arr.length, 0, 'arr.length');
