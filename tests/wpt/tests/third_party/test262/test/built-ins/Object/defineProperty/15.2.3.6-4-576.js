// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-4-576
description: >
    ES5 Attributes - [[Set]] attribute is a function which contains
    return statement
---*/

var obj = {};

var setFunc = function() {
  return 2010;
};

Object.defineProperty(obj, "prop", {
  set: setFunc
});

var desc = Object.getOwnPropertyDescriptor(obj, "prop");

assert(obj.hasOwnProperty("prop"), 'obj.hasOwnProperty("prop") !== true');
assert.sameValue(desc.set, setFunc, 'desc.set');
