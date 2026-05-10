// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-4-341
description: >
    ES5 Attributes - property ([[Writable]] is true, [[Enumerable]] is
    false, [[Configurable]] is true) is non-enumerable
---*/

var obj = {};

Object.defineProperty(obj, "prop", {
  value: 2010,
  writable: true,
  enumerable: false,
  configurable: true
});
var propertyDefineCorrect = obj.hasOwnProperty("prop");
var desc = Object.getOwnPropertyDescriptor(obj, "prop");
for (var p in obj) {
  assert.notSameValue(p, "prop", 'p');
}

assert(propertyDefineCorrect, 'propertyDefineCorrect !== true');
assert.sameValue(desc.enumerable, false, 'desc.enumerable');
