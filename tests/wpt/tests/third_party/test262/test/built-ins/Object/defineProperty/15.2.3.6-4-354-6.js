// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-4-354-6
description: >
    Object.defineProperty - Indexed property 'P' with attributes
    [[Writable]]: false, [[Enumerable]]: true, [[Configurable]]: true
    is non-writable using simple assignment, 'A' is an Array object
includes: [propertyHelper.js]
---*/

var obj = [];

Object.defineProperty(obj, "0", {
  value: 2010,
  writable: false,
  enumerable: true,
  configurable: true
});

verifyProperty(obj, "0", {
  value: 2010,
  writable: false,
});
