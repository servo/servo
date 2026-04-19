// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-4-170
description: >
    Object.defineProperty - 'O' is an Array, 'name' is the length
    property of 'O', the [[Value]] field of 'desc' is less than value
    of  the length property and also lesser than an index of the array
    which is set to configurable: false, test that new length is set
    to a value greater than the non-deletable index by 1, writable
    attribute of length is set to false and TypeError exception is
    thrown (15.4.5.1 step 3.l.ii)
---*/

var arrObj = [0, 1];
assert.throws(TypeError, function() {
  Object.defineProperty(arrObj, "1", {
    configurable: false
  });

  Object.defineProperty(arrObj, "length", {
    value: 1
  });
});
assert.sameValue(arrObj.length, 2, 'arrObj.length');
assert(arrObj.hasOwnProperty("1"), 'arrObj.hasOwnProperty("1") !== true');
