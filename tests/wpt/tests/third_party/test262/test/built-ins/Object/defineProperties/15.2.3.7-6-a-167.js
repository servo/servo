// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.7-6-a-167
description: >
    Object.defineProperties - 'O' is an Array, 'P' is the length
    property of 'O', the [[Value]] field of 'desc' is less than value
    of  the length property, test the [[Configurable]] attribute of
    inherited data property with large index named in 'O' can't stop
    deleting index named properties (15.4.5.1 step 3.l.ii)
---*/

var arr = [0, 1];

Array.prototype[1] = 2; //we are not allowed to set the [[Configurable]] attribute of property "1" to false here, since Array.prototype is a global object, and non-configurbale property can't revert to configurable

Object.defineProperties(arr, {
  length: {
    value: 1
  }
});

assert.sameValue(arr.length, 1, 'arr.length');
assert.sameValue(arr.hasOwnProperty("1"), false, 'arr.hasOwnProperty("1")');
assert.sameValue(arr[0], 0, 'arr[0]');
assert.sameValue(Array.prototype[1], 2, 'Array.prototype[1]');
