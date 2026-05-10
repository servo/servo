// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.lastindexof
description: Array.prototype.lastIndexOf - search element is NaN
---*/

assert.sameValue([+NaN, NaN, -NaN].lastIndexOf(NaN), -1, '[+NaN, NaN, -NaN].lastIndexOf(NaN)');
