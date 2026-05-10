// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.indexof
description: >
    Array.prototype.indexOf - 'length' is a string containing a
    decimal number
---*/

var obj = {
  199: true,
  200: "200.59",
  length: "200.59"
};

assert.sameValue(Array.prototype.indexOf.call(obj, true), 199, 'Array.prototype.indexOf.call(obj, true)');
assert.sameValue(Array.prototype.indexOf.call(obj, "200.59"), -1, 'Array.prototype.indexOf.call(obj, "200.59")');
