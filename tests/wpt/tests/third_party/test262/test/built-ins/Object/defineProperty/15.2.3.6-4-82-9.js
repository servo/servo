// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-4-82-9
description: >
    Object.defineProperty - Update [[Configurable]] attribute of
    'name' property to false successfully when [[Enumerable]] and
    [[Configurable]] attributes of 'name' property are true,  the
    'desc' is a generic descriptor which only contains
    [[Configurable]] attribute as false, 'name' property is an
    accessor property (8.12.9 step 8)
includes: [propertyHelper.js]
---*/


var obj = {};
obj.verifySetFunction = "data";
var get_func = function() {
  return obj.verifySetFunction;
};
var set_func = function(value) {
  obj.verifySetFunction = value;
};
Object.defineProperty(obj, "foo", {
  get: get_func,
  set: set_func,
  enumerable: true,
  configurable: true
});

Object.defineProperty(obj, "foo", {
  configurable: false
});

verifyEqualTo(obj, "foo", get_func());

verifyWritable(obj, "foo", "verifySetFunction");

verifyProperty(obj, "foo", {
  enumerable: true,
  configurable: false,
});
