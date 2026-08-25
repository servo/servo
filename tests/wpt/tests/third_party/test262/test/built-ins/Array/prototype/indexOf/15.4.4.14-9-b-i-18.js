// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.indexof
description: >
    Array.prototype.indexOf - element to be retrieved is own accessor
    property without a get function on an Array-like object
---*/

var obj = {
  length: 1
};
Object.defineProperty(obj, "0", {
  set: function() {},
  configurable: true
});

assert.sameValue(Array.prototype.indexOf.call(obj, undefined), 0, 'Array.prototype.indexOf.call(obj, undefined)');
