// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.14-5-2
description: >
    Object.keys - own enumerable accessor property of 'O' is defined
    in returned array
---*/

var obj = {};

Object.defineProperty(obj, "prop", {
  get: function() {
    return 1003;
  },
  enumerable: true,
  configurable: true
});

var arr = Object.keys(obj);

assert(arr.hasOwnProperty(0), 'arr.hasOwnProperty(0) !== true');
assert.sameValue(arr[0], "prop", 'arr[0]');
