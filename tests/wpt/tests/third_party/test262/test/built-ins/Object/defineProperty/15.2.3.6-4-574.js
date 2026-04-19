// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-4-574
description: >
    ES5 Attributes - [[Set]] attribute is a function which has two
    arguments
---*/

var obj = {};
var firstArg = 12;
var secondArg = 12;

var setFunc = function(a, b) {
  firstArg = a;
  secondArg = b;
};
Object.defineProperty(obj, "prop", {
  set: setFunc
});
obj.prop = 100;
var desc = Object.getOwnPropertyDescriptor(obj, "prop");

assert(obj.hasOwnProperty("prop"), 'obj.hasOwnProperty("prop") !== true');
assert.sameValue(desc.set, setFunc, 'desc.set');
assert.sameValue(firstArg, 100, 'firstArg');
assert.sameValue(typeof secondArg, "undefined", 'typeof secondArg');
