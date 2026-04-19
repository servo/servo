// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.indexof
description: >
    Array.prototype.indexOf - both array element and search element
    are Number, and they have same value
---*/

assert.sameValue([-1, 0, 1].indexOf(1), 2, '[-1, 0, 1].indexOf(1)');
