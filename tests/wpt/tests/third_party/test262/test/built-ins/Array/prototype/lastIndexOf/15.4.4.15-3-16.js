// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.lastindexof
description: >
    Array.prototype.lastIndexOf - value of 'length' is a string which
    is able to be converted into hex number
---*/

var obj = {
  2573: 2573,
  2574: 0x000A0E,
  length: "0x000A0E"
};

assert.sameValue(Array.prototype.lastIndexOf.call(obj, 2573), 2573, 'Array.prototype.lastIndexOf.call(obj, 2573)');
assert.sameValue(Array.prototype.lastIndexOf.call(obj, 0x000A0E), -1, 'Array.prototype.lastIndexOf.call(obj, 0x000A0E)');
