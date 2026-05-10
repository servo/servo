// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.7-6-a-186
description: >
    Object.defineProperties - 'O' is an Array, 'P' is an array index
    named property, 'P' is own data property  (15.4.5.1 step 4.c)
---*/

var arr = [];
Object.defineProperty(arr, 0, {
  value: "ownDataProperty",
  configurable: false
});
assert.throws(TypeError, function() {
  Object.defineProperties(arr, {
    "0": {
      value: "abc",
      configurable: true
    }
  });
});
assert.sameValue(arr[0], "ownDataProperty", 'arr[0]');
