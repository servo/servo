// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-4-582
description: >
    ES5 Attributes - Inherited property is non-enumerable (Function
    instance)
---*/

var data = "data";

Object.defineProperty(Function.prototype, "prop", {
  get: function() {
    return data;
  },
  enumerable: false,
  configurable: true
});
var funObj = function() {};
var verifyEnumerable = false;
for (var p in funObj) {
  if (p === "prop") {
    verifyEnumerable = true;
  }
}

assert.sameValue(funObj.hasOwnProperty("prop"), false, 'funObj.hasOwnProperty("prop")');
assert.sameValue(verifyEnumerable, false, 'verifyEnumerable');
