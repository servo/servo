// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-4-195
description: >
    Object.defineProperty - 'O' is an Array, 'name' is an array index
    named property, 'name' is an inherited accessor property (15.4.5.1
    step 4.c)
includes: [propertyHelper.js]
---*/

function getFunc() {
  return arrObj.helpVerifySet;
}

function setFunc(value) {
  arrObj.helpVerifySet = value;
}

try {
  Object.defineProperty(Array.prototype, "0", {
    get: function() {
      return 11;
    },
    configurable: true
  });

  var arrObj = [];


  Object.defineProperty(arrObj, "0", {
    get: getFunc,
    set: setFunc,
    configurable: false
  });

  arrObj[0] = 13;

  verifyEqualTo(arrObj, "0", getFunc());

  verifyWritable(arrObj, "0", "helpVerifySet");

  verifyProperty(arrObj, "0", {
    enumerable: false,
    configurable: false,
  });
} finally {
  delete Array.prototype[0];
}
