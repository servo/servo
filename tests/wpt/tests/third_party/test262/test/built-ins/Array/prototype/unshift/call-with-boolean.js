// Copyright (c) 2020 Rick Waldron.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.unshift
description: Array.prototype.unshift applied to boolean primitive
---*/

assert.sameValue(Array.prototype.unshift.call(true), 0, 'Array.prototype.unshift.call(true) must return 0');
assert.sameValue(Array.prototype.unshift.call(false), 0, 'Array.prototype.unshift.call(false) must return 0');
