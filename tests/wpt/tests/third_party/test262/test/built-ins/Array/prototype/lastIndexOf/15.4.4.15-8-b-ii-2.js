// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.lastindexof
description: >
    Array.prototype.lastIndexOf - both type of array element and type
    of search element are Undefined
---*/

assert.sameValue([undefined].lastIndexOf(), 0, '[undefined].lastIndexOf()');
assert.sameValue([undefined].lastIndexOf(undefined), 0, '[undefined].lastIndexOf(undefined)');
