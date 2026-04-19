// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.7-6-a-163
description: >
    Object.defineProperties - 'O' is an Array, 'P' is the length
    property of 'O', the [[Value]] field of 'desc' is less than value
    of  the length property, test the [[Writable]] attribute of the
    length property is set to false at last after deleting properties
    with large index named if the [[Writable]] field of 'desc' is
    false (15.4.5.1 step 3.i.ii)
includes: [propertyHelper.js]
---*/


var arr = [0, 1];

Object.defineProperties(arr, {
  length: {
    value: 1,
    writable: false
  }
});

//try to overwrite length value of arr
verifyProperty(arr, "length", {
  value: 1,
  writable: false,
});

assert(!arr.hasOwnProperty("1"));
assert.sameValue(arr[0], 0);
