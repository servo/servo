// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-4-580
description: >
    ES5 Attributes - Inherited property is enumerable (Boolean
    instance)
---*/

var data = "data";

Object.defineProperty(Boolean.prototype, "prop", {
  get: function() {
    return data;
  },
  set: function(value) {
    data = value;
  },
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
