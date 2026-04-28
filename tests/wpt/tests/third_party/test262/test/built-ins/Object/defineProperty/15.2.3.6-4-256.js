// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-4-256
description: >
    Object.defineProperty - 'O' is an Array, 'name' is an array index
    named property, 'name' is accessor property and 'desc' is accessor
    descriptor, and the [[Configurable]] attribute value of 'name' is
    false, test TypeError is thrown if the [[Get]] field of 'desc' is
    present, and the [[Get]] field of 'desc' is an object and the
    [[Get]] attribute value of 'name' is undefined (15.4.5.1 step 4.c)
includes: [propertyHelper.js]
---*/

var arrObj = [];

function getFunc() {
  return 12;
}

Object.defineProperty(arrObj, "1", {
  get: getFunc
});

try {
  Object.defineProperty(arrObj, "1", {
    get: undefined
  });
  throw new Test262Error("Expected TypeError");
} catch (e) {
  assert(e instanceof TypeError);
  assert(arrObj.hasOwnProperty("1"));

  var desc = Object.getOwnPropertyDescriptor(arrObj, "1");

  assert(arrObj[1] === getFunc());
  assert(desc.hasOwnProperty("set") && typeof desc.set === "undefined");

  verifyNotWritable(arrObj, "1");
}

verifyProperty(arrObj, "1", {
  configurable: false,
});
