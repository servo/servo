// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-4-567
description: >
    ES5 Attributes - [[Get]] attribute is a function which has one
    argument
---*/

var obj = {};
var getFunc = function(arg1) {
  return 2010;
};

Object.defineProperty(obj, "prop", {
  get: getFunc
});

var desc = Object.getOwnPropertyDescriptor(obj, "prop");

assert(obj.hasOwnProperty("prop"), 'obj.hasOwnProperty("prop") !== true');
assert.sameValue(obj.prop, 2010, 'obj.prop');
assert.sameValue(desc.get, getFunc, 'desc.get');
