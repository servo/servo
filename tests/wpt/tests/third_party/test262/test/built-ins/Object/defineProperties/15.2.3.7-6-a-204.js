// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.7-6-a-204
description: >
    Object.defineProperties - 'O' is an Array, 'P' is an array index
    named property, 'P' property doesn't exist in 'O', test
    [[Configurable]] of 'P' property in 'Attributes' is set as false
    value if [[Configurable]] is absent in accessor descriptor 'desc'
    (15.4.5.1 step 4.c)
includes: [propertyHelper.js]
---*/

var arr = [];
arr.verifySetter = 100;

Object.defineProperties(arr, {
  "0": {
    set: function(value) {
      arr.verifySetter = value;
    },
    get: function() {
      return arr.verifySetter;
    },
    enumerable: true
  }
});

if (!Object.prototype.hasOwnProperty.call(arr, "0")) {
  throw new Test262Error("Expected hasOwnProperty to return true.");
}

arr[0] = 101;

verifyEqualTo(arr, 0, 101);

if (arr.verifySetter !== 101) {
  throw new Test262Error('Expected arr.verifySetter === 101, actually ' + arr.verifySetter);
}

verifyProperty(arr, "0", {
  configurable: false,
});
