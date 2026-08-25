// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.14-3-7
description: >
    Object.keys - length of the returned array equals the number of
    own enumerable properties of 'O'
---*/

var obj = {
  prop1: 1001,
  prop2: 1002
};

Object.defineProperty(obj, "prop3", {
  value: 1003,
  enumerable: true,
  configurable: true
});

Object.defineProperty(obj, "prop4", {
  get: function() {
    return 1003;
  },
  enumerable: false,
  configurable: true
});

var arr = Object.keys(obj);

assert.sameValue(arr.length, 3, 'arr.length');
