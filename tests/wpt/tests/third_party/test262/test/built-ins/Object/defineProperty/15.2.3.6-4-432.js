// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-4-432
description: >
    ES5 Attributes - [[Set]] attribute of accessor property ([[Get]]
    is undefined, [[Set]] is undefined, [[Enumerable]] is true,
    [[Configurable]] is false) is undefined
---*/

var obj = {};

Object.defineProperty(obj, "prop", {
  get: undefined,
  set: undefined,
  enumerable: true,
  configurable: false
});

var propertyDefineCorrect = obj.hasOwnProperty("prop");
var desc = Object.getOwnPropertyDescriptor(obj, "prop");

assert(propertyDefineCorrect, 'propertyDefineCorrect !== true');
assert.sameValue(typeof desc.set, "undefined", 'typeof desc.set');
