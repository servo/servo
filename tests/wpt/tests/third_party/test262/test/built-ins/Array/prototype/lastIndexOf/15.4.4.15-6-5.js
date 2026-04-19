// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.lastindexof
description: Array.prototype.lastIndexOf returns -1 when 'fromIndex' is 1
---*/

assert.sameValue([1, 2, 3].lastIndexOf(3, 1), -1, '[1, 2, 3].lastIndexOf(3, 1)');
