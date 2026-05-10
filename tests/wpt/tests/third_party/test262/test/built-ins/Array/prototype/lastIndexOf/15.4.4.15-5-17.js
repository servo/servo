// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.lastindexof
description: >
    Array.prototype.lastIndexOf - value of 'fromIndex' is a string
    containing -Infinity
---*/

assert.sameValue([true].lastIndexOf(true, "-Infinity"), -1, '[true].lastIndexOf(true, "-Infinity")');
