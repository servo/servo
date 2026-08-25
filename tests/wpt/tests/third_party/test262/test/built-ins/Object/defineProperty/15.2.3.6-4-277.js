// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-4-277
description: >
    Object.defineProperty -  'O' is an Array, 'name' is generic
    property that won't exist on 'O', and 'desc' is data descriptor,
    test 'name' is defined in 'O' with all correct attribute values
    (15.4.5.1 step 5)
includes: [propertyHelper.js]
---*/


var arrObj = [];

Object.defineProperty(arrObj, "property", {
  value: 12,
  writable: true,
  enumerable: true,
  configurable: true
});

verifyProperty(arrObj, "property", {
  value: 12,
  writable: true,
  enumerable: true,
  configurable: true,
});
