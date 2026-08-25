// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.reduceright
description: >
    Array.prototype.reduceRight - 'initialValue' is returned if 'len'
    is 0 and 'initialValue' is present
---*/

var initialValue = 10;

assert.sameValue([].reduceRight(function() {}, initialValue), initialValue, '[].reduceRight(function () { }, initialValue)');
