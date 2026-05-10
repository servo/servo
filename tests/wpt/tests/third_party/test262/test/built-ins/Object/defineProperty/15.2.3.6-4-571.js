// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-4-571
description: >
    ES5 Attributes - [[Get]] attribute is a function which involves
    'this' object into statement(s)
---*/

var obj = {
  len: 2010
};
var getFunc = function() {
  return this;
};

Object.defineProperty(obj, "prop", {
  get: getFunc
});

var desc = Object.getOwnPropertyDescriptor(obj, "prop");

assert(obj.hasOwnProperty("prop"), 'obj.hasOwnProperty("prop") !== true');
assert.sameValue(obj.prop, obj, 'obj.prop');
assert.sameValue(desc.get, getFunc, 'desc.get');
