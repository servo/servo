// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.9-2-b-i-1
description: >
    Object.freeze - The [[Wrtiable]] attribute of own data property of
    'O' is set to false while other attributes are unchanged
includes: [propertyHelper.js]
---*/

var obj = {};

Object.defineProperty(obj, "foo", {
  value: 10,
  writable: true,
  enumerable: true,
  configurable: false
});

Object.freeze(obj);

verifyProperty(obj, "foo", {
  value: 10,
  writable: false,
  enumerable: true,
  configurable: false,
});
