// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-4-182
description: >
    Object.defineProperty - 'O' is an Array, 'name' is an array index
    named property, 'name' is available String values that convert to
    numbers (15.4.5.1 step 4.a)
---*/

var arrObj = [];

Object.defineProperty(arrObj, "0", {
  value: 12
});

assert.sameValue(arrObj[0], 12, 'arrObj[0]');
