// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.map
description: >
    Array.prototype.map - deleting own property with prototype
    property causes prototype index property to be visited on an Array
---*/

function callbackfn(val, idx, obj) {
  if (idx === 1 && val === 3) {
    return false;
  } else {
    return true;
  }
}
var arr = [0, 1, 2];

Object.defineProperty(arr, "0", {
  get: function() {
    delete arr[1];
    return 0;
  },
  configurable: true
});

Array.prototype[1] = 3;
var testResult = arr.map(callbackfn);

assert.sameValue(testResult[1], false, 'testResult[1]');
