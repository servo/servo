// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-4-165
description: >
    Object.defineProperty - 'O' is an Array, 'name' is the length
    property of 'O', the [[Value]] field of 'desc' is less than value
    of  the length property,  test the [[Writable]] attribute of the
    length property is set to true after deleting properties with
    large index named if the [[Writable]] field of 'desc' is absent
    (15.4.5.1 step 3.h)
---*/

var arrObj = [0, 1];

Object.defineProperty(arrObj, "length", {
  value: 1
});

var indexDeleted = !arrObj.hasOwnProperty("1");

arrObj.length = 10;

assert(indexDeleted, 'indexDeleted !== true');
assert.sameValue(arrObj.length, 10, 'arrObj.length');
