// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.lastindexof
description: >
    Array.prototype.lastIndexOf - element to be retrieved is inherited
    accessor property on an Array
---*/

Object.defineProperty(Array.prototype, "0", {
  get: function() {
    return 10;
  },
  configurable: true
});

Object.defineProperty(Array.prototype, "1", {
  get: function() {
    return 20;
  },
  configurable: true
});

Object.defineProperty(Array.prototype, "2", {
  get: function() {
    return 30;
  },
  configurable: true
});

assert.sameValue([, , , ].lastIndexOf(10), 0, '[, , , ].lastIndexOf(10)');
assert.sameValue([, , , ].lastIndexOf(20), 1, '[, , , ].lastIndexOf(20)');
assert.sameValue([, , , ].lastIndexOf(30), 2, '[, , , ].lastIndexOf(30)');
