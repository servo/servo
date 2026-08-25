// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-4-243-1
description: >
    Object.defineProperty - 'O' is an Array, 'name' is an array index
    named property,  'name' is accessor property and  assignment to
    the accessor property, fails to convert accessor property from
    accessor property to data property (15.4.5.1 step 4.c)
includes: [propertyHelper.js]
flags: [noStrict]
---*/


var arrObj = [];

function getFunc() {
  return 3;
}
Object.defineProperty(arrObj, "1", {
  get: getFunc,
  configurable: true
});

arrObj[1] = 4;

verifyEqualTo(arrObj, "1", getFunc());

verifyProperty(arrObj, "1", {
  enumerable: false,
  configurable: true,
});
