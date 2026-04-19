// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.7-6-a-193
description: >
    Object.defineProperties - 'O' is an Array, 'P' is an array index
    named property, 'P' is own accessor property that overrides an
    inherited accessor property  (15.4.5.1 step 4.c)
---*/

var arr = [];

assert.throws(TypeError, function() {
  Object.defineProperty(Array.prototype, "0", {
    get: function() {
      return 11;
    },
    configurable: true
  });

  Object.defineProperty(arr, "0", {
    get: function() {
      return 12;
    },
    configurable: false
  });

  Object.defineProperties(arr, {
    "0": {
      configurable: true
    }
  });
});
assert.sameValue(arr[0], 12, 'arr[0]');
assert.sameValue(Array.prototype[0], 11, 'Array.prototype[0]');
