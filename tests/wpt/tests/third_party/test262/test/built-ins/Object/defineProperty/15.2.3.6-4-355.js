// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-4-355
description: >
    ES5 Attributes - property ([[Writable]] is false, [[Enumerable]]
    is true, [[Configurable]] is true) is enumerable
---*/

var propertyFound = false;

var obj = {};

Object.defineProperty(obj, "prop", {
  value: 2010,
  writable: false,
  enumerable: true,
  configurable: true
});
var propertyDefineCorrect = obj.hasOwnProperty("prop");
var desc = Object.getOwnPropertyDescriptor(obj, "prop");
for (var property in obj) {
  if (property === "prop") {
    propertyFound = true;
    break;
  }
}

assert(propertyFound, 'Property not found');
assert(propertyDefineCorrect, 'propertyDefineCorrect !== true');
assert.sameValue(desc.enumerable, true, 'desc.enumerable');
