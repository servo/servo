// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.indexof
description: Array.prototype.indexOf - 'length' is own accessor property
---*/

var objOne = {
  1: true
};
var objTwo = {
  2: true
};
Object.defineProperty(objOne, "length", {
  get: function() {
    return 2;
  },
  configurable: true
});
Object.defineProperty(objTwo, "length", {
  get: function() {
    return 2;
  },
  configurable: true
});

assert.sameValue(Array.prototype.indexOf.call(objOne, true), 1, 'Array.prototype.indexOf.call(objOne, true)');
assert.sameValue(Array.prototype.indexOf.call(objTwo, true), -1, 'Array.prototype.indexOf.call(objTwo, true)');
