// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-4-573
description: >
    ES5 Attributes - [[Set]] attribute is a function which has one
    argument
---*/

var obj = {};

var verifySetFunc = 20;
var setFunc = function(value) {
  verifySetFunc = value;
};
Object.defineProperty(obj, "prop", {
  set: setFunc
});
obj.prop = 2010;
var desc = Object.getOwnPropertyDescriptor(obj, "prop");

assert(obj.hasOwnProperty("prop"), 'obj.hasOwnProperty("prop") !== true');
assert.sameValue(desc.set, setFunc, 'desc.set');
assert.sameValue(verifySetFunc, 2010, 'verifySetFunc');
