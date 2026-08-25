// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.7-6-a-119
description: >
    Object.defineProperties - 'O' is an Array, 'P' is the length
    property of 'O', the [[Value]] field of 'desc' is absent, test
    TypeError is thrown when updating the [[Writable]] attribute of
    the length property from false to true (15.4.5.1 step 3.a.i)
---*/

var arr = [];

Object.defineProperty(arr, "length", {
  writable: false
});
assert.throws(TypeError, function() {
  Object.defineProperties(arr, {
    length: {
      writable: true
    }
  });
});
