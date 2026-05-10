// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.14-2-7
description: >
    Object.keys - 'n' is 0 when 'O' doesn't contain own enumerable
    data or accessor properties
---*/

var obj = {};

Object.defineProperty(obj, "prop1", {
  value: 1001,
  enumerable: false,
  configurable: true
});

Object.defineProperty(obj, "prop2", {
  get: function() {
    return 1002;
  },
  enumerable: false,
  configurable: true
});

var arr = Object.keys(obj);

assert.sameValue(arr.length, 0, 'arr.length');
