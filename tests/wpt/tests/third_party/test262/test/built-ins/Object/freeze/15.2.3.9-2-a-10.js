// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.9-2-a-10
description: >
    Object.freeze - 'P' is own named property of an Array object that
    uses Object's [[GetOwnProperty]]
includes: [propertyHelper.js]
---*/

var arrObj = [];

arrObj.foo = 10; // default [[Configurable]] attribute value of foo: true

Object.freeze(arrObj);

verifyProperty(arrObj, "foo", {
  value: 10,
  writable: false,
  configurable: false,
});
