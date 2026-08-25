// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-4-523
description: >
    ES5 Attributes - property ([[Get]] is a Function, [[Set]] is
    undefined, [[Enumerable]] is false, [[Configurable]] is false) is
    non-enumerable
---*/

var obj = {};

var getFunc = function() {
  return 1001;
};

Object.defineProperty(obj, "prop", {
  get: getFunc,
  set: undefined,
  enumerable: false,
  configurable: false
});

var propertyDefineCorrect = obj.hasOwnProperty("prop");
var desc = Object.getOwnPropertyDescriptor(obj, "prop");

for (var p in obj) {
  assert.notSameValue(p, "prop", 'p');
}

assert(propertyDefineCorrect, 'propertyDefineCorrect !== true');
assert.sameValue(desc.enumerable, false, 'desc.enumerable');
