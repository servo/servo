// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.9-2-b-i-2
description: >
    Object.freeze - The [[Wrtiable]] attribute of all own data
    property of 'O' is set to false while other attributes are
    unchanged
includes: [propertyHelper.js]
---*/

var obj = {};

Object.defineProperty(obj, "foo1", {
  value: 10,
  writable: false,
  enumerable: true,
  configurable: false
});

Object.defineProperty(obj, "foo2", {
  value: 20,
  writable: true,
  enumerable: false,
  configurable: false
});

Object.freeze(obj);

verifyProperty(obj, "foo1", {
  value: 10,
  writable: false,
  enumerable: true,
  configurable: false,
});

verifyProperty(obj, "foo2", {
  value: 20,
  writable: false,
  enumerable: false,
  configurable: false,
});
