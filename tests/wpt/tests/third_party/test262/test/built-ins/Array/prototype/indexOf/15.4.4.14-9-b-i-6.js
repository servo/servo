// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.indexof
description: >
    Array.prototype.indexOf - element to be retrieved is own data
    property that overrides an inherited accessor property on an
    Array-like object
---*/

Object.defineProperty(Object.prototype, "0", {
  get: function() {
    return false;
  },
  configurable: true
});

assert.sameValue(Array.prototype.indexOf.call({
  0: true,
  1: 1,
  length: 2
}, true), 0, 'Array.prototype.indexOf.call({ 0: true, 1: 1, length: 2 }, true)');
