// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.indexof
description: Array.prototype.indexOf returns -1 if 'length' is 0 (empty array)
---*/

var i = [].indexOf(42);

assert.sameValue(i, -1, 'i');
