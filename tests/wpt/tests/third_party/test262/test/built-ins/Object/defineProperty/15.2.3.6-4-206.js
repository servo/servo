// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-4-206
description: >
    Object.defineProperty - 'O' is an Array, 'name' is an array index
    named property, 'name' property doesn't exist in 'O', test [[Set]]
    of 'name' property in 'Attributes' is set as undefined if [[Set]]
    is absent in accessor descriptor 'desc' (15.4.5.1 step 4.c)
---*/

var arrObj = [];
var getFunc = function() {};

Object.defineProperty(arrObj, "0", {
  get: getFunc,
  enumerable: true,
  configurable: true
});

var desc = Object.getOwnPropertyDescriptor(arrObj, "0");

assert(arrObj.hasOwnProperty("0"), 'arrObj.hasOwnProperty("0") !== true');
assert(desc.hasOwnProperty("set"), 'desc.hasOwnProperty("set") !== true');
assert.sameValue(typeof desc.set, "undefined", 'typeof desc.set');
