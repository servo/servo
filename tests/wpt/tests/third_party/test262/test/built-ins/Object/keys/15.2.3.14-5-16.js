// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.14-5-16
description: >
    Object.keys - own enumerable indexed accessor property of String
    object 'O' is defined in returned array
---*/

var obj = new String("xyz");
obj[-20] = -20;
obj[20] = 20;

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

var arr = Object.keys(obj);

for (var i = 0; i < arr.length; i++) {
  assert(obj.hasOwnProperty(arr[i]), 'obj.hasOwnProperty(arr[i]) !== true');
}
