// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-4-265
description: >
    Object.defineProperty - 'O' is an Array, 'name' is an array index
    named property, name is accessor property and 'desc' is accessor
    descriptor, test updating the [[Get]] attribute value of 'name'
    (15.4.5.1 step 4.c)
includes: [propertyHelper.js]
---*/


var arrObj = [];

function getFunc() {
  return 100;
}
Object.defineProperty(arrObj, "0", {
  get: function() {
    return 12;
  },
  configurable: true
});
Object.defineProperty(arrObj, "0", {
  get: getFunc
});
verifyEqualTo(arrObj, "0", getFunc());

verifyProperty(arrObj, "0", {
  enumerable: false,
  configurable: true,
});
