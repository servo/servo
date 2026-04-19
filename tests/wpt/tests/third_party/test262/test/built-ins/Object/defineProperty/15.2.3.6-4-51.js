// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-4-51
description: >
    Object.defineProperty - desc is data descriptor, test updating all
    attribute values of 'name' (8.12.9 step 4.a.i)
includes: [propertyHelper.js]
---*/

var obj = {
  "property": 1
}; // default value of attributes: writable: true, configurable: true, enumerable: true

Object.defineProperty(obj, "property", {
  value: 1001,
  writable: false,
  enumerable: false,
  configurable: false
});

verifyProperty(obj, "property", {
  value: 1001,
  writable: false,
  enumerable: false,
  configurable: false,
});
