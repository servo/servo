// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.indexof
description: >
    Array.prototype.indexOf - element to be retrieved is inherited
    accessor property without a get function on an Array
---*/

Object.defineProperty(Array.prototype, "0", {
  set: function() {},
  configurable: true
});

assert.sameValue([, ].indexOf(undefined), 0, '[, ].indexOf(undefined)');
