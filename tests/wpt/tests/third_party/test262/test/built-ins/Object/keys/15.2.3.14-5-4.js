// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.14-5-4
description: >
    Object.keys - non-enumerable own accessor property of 'O' is not
    defined in returned array
---*/

var obj = {};

Object.defineProperty(obj, "prop1", {
  get: function() {},
  enumerable: true,
  configurable: true
});
Object.defineProperty(obj, "prop2", {
  get: function() {},
  enumerable: false,
  configurable: true
});
Object.defineProperty(obj, "prop3", {
  get: function() {},
  enumerable: true,
  configurable: true
});

var arr = Object.keys(obj);

for (var p in arr) {
  if (arr.hasOwnProperty(p)) {
    assert.notSameValue(arr[p], "prop2", 'arr[p]');
  }
}
