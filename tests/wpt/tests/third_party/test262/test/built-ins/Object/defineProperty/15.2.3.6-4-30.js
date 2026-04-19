// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-4-30
description: >
    Object.defineProperty - 'name' is own accessor property without a
    get function (8.12.9 step 1)
---*/

var obj = {};
Object.defineProperty(obj, "foo", {
  set: function() {},
  configurable: false
});
assert.throws(TypeError, function() {
  Object.defineProperty(obj, "foo", {
    configurable: true
  });
});
