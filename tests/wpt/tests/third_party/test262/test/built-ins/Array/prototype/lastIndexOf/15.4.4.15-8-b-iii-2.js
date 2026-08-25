// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.lastindexof
description: >
    Array.prototype.lastIndexOf returns without visiting subsequent
    element once search value is found
---*/

var arr = [2, 1, , 1, 2];
var elementFirstAccessed = false;
var elementThirdAccessed = false;

Object.defineProperty(arr, "2", {
  get: function() {
    elementThirdAccessed = true;
    return 2;
  },
  configurable: true
});
Object.defineProperty(arr, "0", {
  get: function() {
    elementFirstAccessed = true;
    return 2;
  },
  configurable: true
});

arr.lastIndexOf(2);

assert.sameValue(elementThirdAccessed, false, 'elementThirdAccessed');
assert.sameValue(elementFirstAccessed, false, 'elementFirstAccessed');
