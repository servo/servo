// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.lastindexof
description: >
    Array.prototype.lastIndexOf - both array element and search
    element are booleans, and they have same value
---*/

assert.sameValue([false, true].lastIndexOf(true), 1, '[false, true].lastIndexOf(true)');
