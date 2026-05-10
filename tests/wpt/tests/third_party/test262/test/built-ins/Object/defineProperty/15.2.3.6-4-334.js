// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-4-334
description: >
    ES5 Attributes - property ([[Writable]] is true, [[Enumerable]] is
    true, [[Configurable]] is false) is enumerable
---*/

var propertyFound = false;

var obj = {};

Object.defineProperty(obj, "prop", {
  value: 2010,
  writable: true,
  enumerable: true,
  configurable: false
});
var propertyDefineCorrect = obj.hasOwnProperty("prop");
var desc = Object.getOwnPropertyDescriptor(obj, "prop");
for (var p in obj) {
  if (p === "prop") {
    propertyFound = true;
    break;
  }
}

assert(propertyFound, 'Property not found');
assert(propertyDefineCorrect, 'propertyDefineCorrect !== true');
assert.sameValue(desc.enumerable, true, 'desc.enumerable');
