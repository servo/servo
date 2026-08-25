// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.reduceright
description: >
    Array.prototype.reduceRight returns initialValue if 'length' is 0
    and initialValue is present (empty array)
---*/

function cb() {}
assert.sameValue([].reduceRight(cb, 1), 1, '[].reduceRight(cb,1)');
