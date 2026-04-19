// Copyright (c) 2020 Rick Waldron.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.indexof
description: Array.prototype.indexOf applied to boolean primitive
---*/

assert.sameValue(Array.prototype.indexOf.call(true), -1, 'Array.prototype.indexOf.call(true) must return -1');
assert.sameValue(Array.prototype.indexOf.call(false), -1, 'Array.prototype.indexOf.call(false) must return -1');
