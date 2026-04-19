// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-4-116
description: >
    Object.defineProperty - 'O' is an Array, test the length property
    of 'O' is own data property (15.4.5.1 step 1)
includes: [propertyHelper.js]
---*/

var arrObj = [0, 1];
Object.defineProperty(arrObj, "1", {
  value: 1,
  configurable: false
});

assert.throws(TypeError, function() {
  Object.defineProperty(arrObj, "length", {
    value: 1
  });
});

verifyProperty(arrObj, "length", {
  value: 2,
  writable: true,
  configurable: false,
  enumerable: false,
});
