// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.7-6-a-176
description: >
    Object.defineProperties - 'O' is an Array, 'P' is the length
    property of 'O', the [[Value]] field of 'desc' is less than value
    of  the length property, test the [[Writable]] attribute of the
    length property is set to false at last when the [[Writable]]
    field of 'desc' is false and 'O' contains non-configurable large
    index named property (15.4.5.1 step 3.l.iii.2)
includes: [propertyHelper.js]
---*/


var arr = [0, 1];

try {
  Object.defineProperty(arr, "1", {
    configurable: false
  });

  Object.defineProperties(arr, {
    length: {
      value: 1,
      writable: false
    }
  });

  throw new Test262Error("Expected to throw TypeError");
} catch (e) {
  assert(e instanceof TypeError);
}

assert(arr.hasOwnProperty("1"));

verifyProperty(arr, "length", {
  value: 2,
  writable: false,
});

assert.sameValue(arr[0], 0);
assert.sameValue(arr[1], 1);
