// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.reduce
description: >
    Array.prototype.reduce returns initialValue if 'length' is 0 and
    initialValue is present (empty array)
---*/

function cb() {}
assert.sameValue([].reduce(cb, 1), 1, '[].reduce(cb,1)');
