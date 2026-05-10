// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-4-197
description: >
    Object.defineProperty - 'O' is an Array, 'name' is an array index
    named property, 'name' is own accessor property that overrides an
    inherited accessor property (15.4.5.1 step 4.c)
---*/


assert.throws(TypeError, function() {
  Object.defineProperty(Array.prototype, "0", {
    get: function() {},
    configurable: true
  });

  var arrObj = [];
  Object.defineProperty(arrObj, "0", {
    get: function() {},
    configurable: false
  });

  Object.defineProperty(arrObj, "0", {
    configurable: true
  });
});
