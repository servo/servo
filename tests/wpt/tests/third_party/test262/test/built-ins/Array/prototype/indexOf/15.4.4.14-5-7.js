// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.indexof
description: >
    Array.prototype.indexOf - value of 'fromIndex' is a number (value
    is 0)
---*/

assert.sameValue([true].indexOf(true, 0), 0, '[true].indexOf(true, 0)');
