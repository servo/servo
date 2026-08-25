// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.indexof
description: >
    Array.prototype.indexOf - element to be retrieved is inherited
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

assert.sameValue([, , , ].indexOf(10), 0, '[, , , ].indexOf(10)');
assert.sameValue([, , , ].indexOf(20), 1, '[, , , ].indexOf(20)');
assert.sameValue([, , , ].indexOf(30), 2, '[, , , ].indexOf(30)');
