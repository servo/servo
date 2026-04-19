// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.lastindexof
description: >
    Array.prototype.lastIndexOf - value of 'length' is a string
    containing an exponential number
---*/

var obj = {
  229: 229,
  230: 2.3E2,
  length: "2.3E2"
};

assert.sameValue(Array.prototype.lastIndexOf.call(obj, 229), 229, 'Array.prototype.lastIndexOf.call(obj, 229)');
assert.sameValue(Array.prototype.lastIndexOf.call(obj, 2.3E2), -1, 'Array.prototype.lastIndexOf.call(obj, 2.3E2)');
