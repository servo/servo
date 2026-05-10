// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.7-6-a-191
description: >
    Object.defineProperties - 'O' is an Array, 'P' is an array index
    property, 'P' is inherited accessor property  (15.4.5.1 step 4.c)
---*/

Object.defineProperty(Array.prototype, "0", {
  get: function() {
    return 11;
  },
  configurable: true
});

var arr = [];

Object.defineProperties(arr, {
  "0": {
    get: function() {
      return 12;
    },
    configurable: false
  }
});

assert(arr.hasOwnProperty("0"), 'arr.hasOwnProperty("0") !== true');
assert.sameValue(arr[0], 12, 'arr[0]');
assert.sameValue(Array.prototype[0], 11, 'Array.prototype[0]');
