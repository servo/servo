// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.indexof
description: >
    Array.prototype.indexOf - element to be retrieved is own accessor
    property on an Array-like object
---*/

var obj = {
  length: 3
};
Object.defineProperty(obj, "0", {
  get: function() {
    return 0;
  },
  configurable: true
});

Object.defineProperty(obj, "1", {
  get: function() {
    return 1;
  },
  configurable: true
});

Object.defineProperty(obj, "2", {
  get: function() {
    return 2;
  },
  configurable: true
});

assert.sameValue(Array.prototype.indexOf.call(obj, 0), 0, 'Array.prototype.indexOf.call(obj, 0)');
assert.sameValue(Array.prototype.indexOf.call(obj, 1), 1, 'Array.prototype.indexOf.call(obj, 1)');
assert.sameValue(Array.prototype.indexOf.call(obj, 2), 2, 'Array.prototype.indexOf.call(obj, 2)');
