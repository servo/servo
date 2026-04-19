// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-4-105
description: >
    Object.defineProperty - 'name' and 'desc' are data properties,
    name.configurable = true and desc.configurable = false (8.12.9
    step 12)
includes: [propertyHelper.js]
---*/

var obj = {};

Object.defineProperty(obj, "foo", {
  value: 200,
  enumerable: true,
  writable: true,
  configurable: true
});

Object.defineProperty(obj, "foo", {
  configurable: false
});

verifyProperty(obj, "foo", {
  value: 200,
  writable: true,
  enumerable: true,
  configurable: false,
});
