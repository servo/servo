// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.indexof
description: >
    Array.prototype.indexOf - value of 'fromIndex' is a number (value
    is NaN)
---*/

assert.sameValue([true].indexOf(true, NaN), 0, '[true].indexOf(true, NaN)');
assert.sameValue([true].indexOf(true, -NaN), 0, '[true].indexOf(true, -NaN)');
