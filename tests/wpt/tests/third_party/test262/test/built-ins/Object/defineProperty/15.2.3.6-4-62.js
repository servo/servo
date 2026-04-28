// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-4-62
description: >
    Object.defineProperty - both desc.value and name.value are null
    (8.12.9 step 6)
includes: [propertyHelper.js]
---*/


var obj = {};

Object.defineProperty(obj, "foo", {
  value: null
});

Object.defineProperty(obj, "foo", {
  value: null
});

verifyProperty(obj, "foo", {
  value: null,
  writable: false,
  enumerable: false,
  configurable: false,
});
