// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-4-373
description: >
    ES5 Attributes - success to update [[Configurable]] attribute of
    data property ([[Writable]] is false, [[Enumerable]] is false,
    [[Configurable]] is true) to different value
---*/

var obj = {};

Object.defineProperty(obj, "prop", {
  value: 2010,
  writable: false,
  enumerable: false,
  configurable: true
});
var propertyDefineCorrect = obj.hasOwnProperty("prop");
var desc1 = Object.getOwnPropertyDescriptor(obj, "prop");

Object.defineProperty(obj, "prop", {
  configurable: false
});
var desc2 = Object.getOwnPropertyDescriptor(obj, "prop");

assert(propertyDefineCorrect, 'propertyDefineCorrect !== true');
assert.sameValue(desc1.configurable, true, 'desc1.configurable');
assert.sameValue(obj.prop, 2010, 'obj.prop');
assert.sameValue(desc2.configurable, false, 'desc2.configurable');
