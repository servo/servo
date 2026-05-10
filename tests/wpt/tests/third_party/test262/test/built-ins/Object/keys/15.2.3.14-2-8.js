// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.14-2-8
description: >
    Object.keys - 'n' is the correct value when enumerable properties
    exist in 'O'
---*/

var obj = {
  prop1: 1001,
  prop2: function() {
    return 1002;
  }
};

Object.defineProperty(obj, "prop3", {
  value: 1003,
  enumerable: false,
  configurable: true
});

Object.defineProperty(obj, "prop4", {
  get: function() {
    return 1004;
  },
  enumerable: false,
  configurable: true
});

var arr = Object.keys(obj);

assert.sameValue(arr.length, 2, 'arr.length');
assert.sameValue(arr[0], "prop1", 'arr[0]');
assert.sameValue(arr[1], "prop2", 'arr[1]');
