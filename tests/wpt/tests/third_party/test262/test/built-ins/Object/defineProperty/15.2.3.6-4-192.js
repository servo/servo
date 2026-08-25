// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-4-192
description: >
    Object.defineProperty - 'O' is an Array, 'name' is an array index
    named property, 'name' is own data property that overrides an
    inherited data property, test TypeError is thrown on updating the
    [[Configurable]] attribute from false to true (15.4.5.1 step 4.c)
---*/

var arrObj = [];

assert.throws(TypeError, function() {
  Object.defineProperty(Array.prototype, "0", {
    value: 11,
    configurable: true
  });

  Object.defineProperty(arrObj, "0", {
    value: 12,
    configurable: false
  });

  Object.defineProperty(arrObj, "0", {
    configurable: true
  });
});
assert.sameValue(Array.prototype[0], 11, 'Array.prototype[0]');
assert.sameValue(arrObj[0], 12, 'arrObj[0]');
