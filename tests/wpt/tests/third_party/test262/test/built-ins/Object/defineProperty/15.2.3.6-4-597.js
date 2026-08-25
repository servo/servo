// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-4-597
description: >
    ES5 Attributes - Inherited property is non-enumerable
    (Function.prototype.bind)
---*/

var foo = function() {};
var data = "data";

Object.defineProperty(Function.prototype, "prop", {
  get: function() {
    return data;
  },
  enumerable: false,
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
assert.sameValue(verifyEnumerable, false, 'verifyEnumerable');
