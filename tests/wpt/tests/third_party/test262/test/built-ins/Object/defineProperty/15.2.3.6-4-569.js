// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-4-569
description: >
    ES5 Attributes - [[Get]] attribute is a function which contains
    global variable
---*/

var obj = {};
var globalVariable = 20;
var getFunc = function() {
  globalVariable = 2010;
  return globalVariable;
};

Object.defineProperty(obj, "prop", {
  get: getFunc
});
var desc = Object.getOwnPropertyDescriptor(obj, "prop");

assert(obj.hasOwnProperty("prop"), 'obj.hasOwnProperty("prop") !== true');
assert.sameValue(desc.get, getFunc, 'desc.get');
assert.sameValue(obj.prop, 2010, 'obj.prop');
assert.sameValue(globalVariable, 2010, 'globalVariable');
