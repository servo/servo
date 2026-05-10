// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-4-260
description: >
    Object.defineProperty - 'O' is an Array, 'name' is an array index
    named property, name is data property and 'desc' is data
    descriptor, test setting the [[Value]] attribute value of 'name'
    from undefined to number (15.4.5.1 step 4.c)
includes: [propertyHelper.js]
---*/


var arrObj = [undefined];
Object.defineProperty(arrObj, "0", {
  value: 100
});

verifyProperty(arrObj, "0", {
  value: 100,
  writable: true,
  enumerable: true,
  configurable: true,
});
