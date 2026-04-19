// Copyright 2014 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Array.prototype.splice sets `length` on `this`
esid: sec-array.prototype.splice
description: Array.prototype.splice throws if `length` is read-only
---*/

var a = [0, 1, 2];

Object.defineProperty(a, 'length', {
  writable: false
});

assert.throws(TypeError, function() {
  a.splice(1, 2, 4);
});
