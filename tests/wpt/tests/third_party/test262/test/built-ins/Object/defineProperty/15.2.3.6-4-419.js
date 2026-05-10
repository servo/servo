// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-4-419
description: >
    ES5 Attributes - Inherited property whose [[Enumerable]] attribute
    is set to true is enumerable (Function.prototype.bind)
---*/

var foo = function() {};

Object.defineProperty(Function.prototype, "prop", {
  value: 1001,
  writable: true,
  enumerable: true,
  configurable: true
});

var obj = foo.bind({});
var verifyEnumerable = false;
for (var p in obj) {
  if (p === "prop") {
    verifyEnumerable = true;
  }
}

assert.sameValue(obj.hasOwnProperty("prop"), false, 'obj.hasOwnProperty("prop")');
assert(verifyEnumerable, 'verifyEnumerable !== true');
