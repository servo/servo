// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.every
description: Array.prototype.every returns true if 'length' is 0 (empty array)
---*/

function cb() {}
var i = [].every(cb);

assert.sameValue(i, true, 'i');
