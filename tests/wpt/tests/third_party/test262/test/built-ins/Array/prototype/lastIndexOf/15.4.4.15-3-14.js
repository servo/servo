// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.lastindexof
description: >
    Array.prototype.lastIndexOf - value of 'length' is a string
    containing -Infinity
---*/

var objThree = {
  0: true,
  1: true,
  length: "-Infinity"
};

assert.sameValue(Array.prototype.lastIndexOf.call(objThree, true), -1, 'Array.prototype.lastIndexOf.call(objThree, true)');
