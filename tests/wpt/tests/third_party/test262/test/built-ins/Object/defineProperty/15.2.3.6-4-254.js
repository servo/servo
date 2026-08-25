// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-4-254
description: >
    Object.defineProperty - 'O' is an Array, 'name' is an array index
    named property, 'name' is accessor property and 'desc' is accessor
    descriptor, and the [[Configurable]] attribute value of 'name' is
    false, test TypeError is not thrown if the [[Set]] field of 'desc'
    is present, and the [[Set]] field of 'desc' and the [[Set]]
    attribute value of 'name' are undefined (15.4.5.1 step 4.c)
includes: [propertyHelper.js]
---*/

var arrObj = [];

Object.defineProperty(arrObj, "1", {
  set: undefined
});
assert(arrObj.hasOwnProperty("1"));

Object.defineProperty(arrObj, "1", {
  set: undefined
});

var desc = Object.getOwnPropertyDescriptor(arrObj, "1");

assert(desc.hasOwnProperty("get") && typeof desc.get === "undefined");
assert(desc.hasOwnProperty("set") && typeof desc.set === "undefined");

verifyProperty(arrObj, "1", {
  enumerable: false,
  configurable: false,
});
