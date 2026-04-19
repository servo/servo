// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.7-6-a-164
description: >
    Object.defineProperties - 'O' is an Array, 'P' is the length
    property of 'O', the [[Value]] field of 'desc' is less than value
    of the length property, test the [[Writable]] attribute of the
    length property in 'O' is set as true before deleting properties
    with large index named (15.4.5.1 step 3.i.iii)
includes: [propertyHelper.js]
---*/


var arr = [0, 1, 2];
var result = 0;

try {
  Object.defineProperty(arr, "1", {
    configurable: false
  });

  Object.defineProperties(arr, {
    length: {
      value: 0,
      writable: false
    }
  });

  throw new Test262Error("expected to throw TypeError")
} catch (e) {
  assert(e instanceof TypeError);
}

verifyProperty(arr, "length", {
  value: 2,
  writable: false,
});
