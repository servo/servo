// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-4-191
description: >
    Object.defineProperty - 'O' is an Array, 'name' is an array index
    named property, 'name' is an inherited data property, test that
    defining own index named property is successful (15.4.5.1 step 4.c)
---*/

try {
  Object.defineProperty(Array.prototype, "0", {
    value: 11,
    configurable: true
  });

  var arrObj = [];

  Object.defineProperty(arrObj, "0", {
    configurable: false
  });

  if (!arrObj.hasOwnProperty("0")) {
    throw new Test262Error("Expected arrObj.hasOwnProperty('0') === true, actually " + arrObj.hasOwnProperty("0"));
  }
  if (Array.prototype[0] !== 11) {
    throw new Test262Error("Expected Array.prototype[0] === 11), actually " + Array.prototype[0]);
  }
  if (typeof arrObj[0] !== "undefined") {
    throw new Test262Error("Expected typeof arrObj[0] === 'undefined'), actually " + typeof arrObj[0]);
  }

} finally {
  delete Array.prototype[0];
}
