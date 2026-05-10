// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-4-585
description: ES5 Attributes - Inherited property is enumerable (RegExp instance)
---*/

var data = "data";

Object.defineProperty(RegExp.prototype, "prop", {
  get: function() {
    return data;
  },
  set: function(value) {
    data = value;
  },
  enumerable: true,
  configurable: true
});
var regObj = new RegExp();
var verifyEnumerable = false;
for (var p in regObj) {
  if (p === "prop") {
    verifyEnumerable = true;
  }
}

assert.sameValue(regObj.hasOwnProperty("prop"), false, 'regObj.hasOwnProperty("prop")');
assert(verifyEnumerable, 'verifyEnumerable !== true');
