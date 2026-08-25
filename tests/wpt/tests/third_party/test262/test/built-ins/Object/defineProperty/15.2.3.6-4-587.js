// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-4-587
description: ES5 Attributes - Inherited property is non-enumerable (Math)
---*/

var data = "data";

Object.defineProperty(Object.prototype, "prop", {
  get: function() {
    return data;
  },
  enumerable: false,
  configurable: true
});
var verifyEnumerable = false;
for (var p in Math) {
  if (p === "prop") {
    verifyEnumerable = true;
  }
}

assert.sameValue(Math.hasOwnProperty("prop"), false, 'Math.hasOwnProperty("prop")');
assert.sameValue(verifyEnumerable, false, 'verifyEnumerable');
