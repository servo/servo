// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-4-175
description: >
    Object.defineProperty - 'O' is an Array, 'name' is the length
    property of 'O', the [[Value]] field of 'desc' is less than value
    of  the length property, test the [[Configurable]] attribute of an
    inherited accessor property with large index named in 'O' can't
    stop deleting index named properties (15.4.5.1 step 3.l.ii)
---*/

var arrObj = [0, 1];

Object.defineProperty(Array.prototype, "1", {
  get: function() {
    return 1;
  },
  configurable: true // Not setting the [[Configurable]] attribute of property "1" to false here, since Array.prototype is a global object, and non-configurbale property can't revert to configurable
});

Object.defineProperty(arrObj, "length", {
  value: 1
});

assert.sameValue(arrObj.length, 1, 'arrObj.length');
assert.sameValue(arrObj.hasOwnProperty("1"), false, 'arrObj.hasOwnProperty("1")');
