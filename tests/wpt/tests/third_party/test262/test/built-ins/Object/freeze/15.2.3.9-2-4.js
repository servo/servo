// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.9-2-4
description: Object.freeze - Non-enumerable own properties of 'O' are frozen
includes: [propertyHelper.js]
---*/

var obj = {};

Object.defineProperty(obj, "foo", {
  value: 10,
  enumerable: false,
  configurable: true
});

Object.freeze(obj);

verifyProperty(obj, "foo", {
  writable: false,
  configurable: false,
});
