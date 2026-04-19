// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-4-113
description: >
    Object.defineProperty - 'name' and 'desc' are accessor properties,
    name.enumerable and desc.enumerable are different values (8.12.9
    step 12)
includes: [propertyHelper.js]
---*/


var obj = {};

function getFunc() {
  return 10;
}

Object.defineProperty(obj, "foo", {
  get: getFunc,
  enumerable: true,
  configurable: true
});

Object.defineProperty(obj, "foo", {
  get: getFunc,
  enumerable: false
});

verifyEqualTo(obj, "foo", getFunc());

verifyProperty(obj, "foo", {
  enumerable: false,
  configurable: true,
});
