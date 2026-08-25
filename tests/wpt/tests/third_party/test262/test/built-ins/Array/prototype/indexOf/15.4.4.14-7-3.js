// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.indexof
description: >
    Array.prototype.indexOf returns -1 when 'fromIndex' and 'length'
    are both 0
---*/

assert.sameValue([].indexOf(1, 0), -1, '[].indexOf(1, 0)');
