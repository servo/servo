// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.some
description: Array.prototype.some returns false if 'length' is 0 (empty array)
---*/

function cb() {}
var i = [].some(cb);

assert.sameValue(i, false, 'i');
