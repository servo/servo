// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-4-404
description: >
    ES5 Attributes - Inherited property whose [[Enumerable]] attribute
    is set to true is enumerable (Boolean instance)
---*/

Object.defineProperty(Boolean.prototype, "prop", {
  value: 1001,
  writable: true,
  enumerable: true,
  configurable: true
});
var boolObj = new Boolean();

var verifyEnumerable = false;
for (var p in boolObj) {
  if (p === "prop") {
    verifyEnumerable = true;
  }
}

assert.sameValue(boolObj.hasOwnProperty("prop"), false, 'boolObj.hasOwnProperty("prop")');
assert(verifyEnumerable, 'verifyEnumerable !== true');
