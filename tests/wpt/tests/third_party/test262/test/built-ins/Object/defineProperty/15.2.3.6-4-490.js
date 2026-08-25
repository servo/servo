// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-4-490
description: >
    ES5 Attributes - fail to update [[Set]] attribute of accessor
    property ([[Get]] is undefined, [[Set]] is a Function,
    [[Enumerable]] is false, [[Configurable]] is false) to different
    value
---*/

var obj = {};

var verifySetFunc = "data";
var setFunc = function(value) {
  verifySetFunc = value;
};

Object.defineProperty(obj, "prop", {
  get: undefined,
  set: setFunc,
  enumerable: false,
  configurable: false
});

var desc1 = Object.getOwnPropertyDescriptor(obj, "prop");
assert.throws(TypeError, function() {
  Object.defineProperty(obj, "prop", {
    set: undefined
  });
});
var desc2 = Object.getOwnPropertyDescriptor(obj, "prop");

assert.sameValue(desc1.set, setFunc, 'desc1.set');
assert.sameValue(desc2.set, setFunc, 'desc2.set');
