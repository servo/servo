// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-4-82-12
description: >
    Object.defineProperty - Update [[Enumerable]] attributes of 'name'
    property to true successfully when [[Enumerable]] attribute of
    'name' is false and [[Configurable]] attribute of 'name' is true,
    the 'desc' is a generic descriptor which only contains
    [[Enumerable]] attribute as true, 'name' property is an accessor
    property (8.12.9 step 8)
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
  enumerable: false,
  configurable: true
});

Object.defineProperty(obj, "foo", {
  enumerable: true
});

verifyEqualTo(obj, "foo", get_func());

verifyWritable(obj, "foo", "verifySetFunction");

verifyProperty(obj, "foo", {
  enumerable: true,
  configurable: true,
});
