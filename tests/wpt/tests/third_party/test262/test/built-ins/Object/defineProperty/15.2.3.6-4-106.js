// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-4-106
description: >
    Object.defineProperty - 'name' and 'desc' are data properties,
    several attributes values of name and desc are different (8.12.9
    step 12)
includes: [propertyHelper.js]
---*/

var obj = {};

Object.defineProperty(obj, "foo", {
  value: 100,
  writable: true,
  enumerable: true,
  configurable: true
});

Object.defineProperty(obj, "foo", {
  value: 200,
  writable: false,
  enumerable: false
});

verifyProperty(obj, "foo", {
  value: 200,
  writable: false,
  enumerable: false,
  configurable: true,
});
