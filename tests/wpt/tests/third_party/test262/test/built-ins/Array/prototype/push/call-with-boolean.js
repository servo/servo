// Copyright (c) 2020 Rick Waldron.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.push
description: Array.prototype.push applied to boolean primitive
---*/

assert.sameValue(Array.prototype.push.call(true), 0, 'Array.prototype.push.call(true) must return 0');
assert.sameValue(Array.prototype.push.call(false), 0, 'Array.prototype.push.call(false) must return 0');
