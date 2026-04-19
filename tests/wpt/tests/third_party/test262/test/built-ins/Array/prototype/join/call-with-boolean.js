// Copyright (c) 2020 Rick Waldron.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.join
description: Array.prototype.join applied to boolean primitive
---*/

assert.sameValue(Array.prototype.join.call(true), "", 'Array.prototype.join.call(true) must return ""');
assert.sameValue(Array.prototype.join.call(false), "", 'Array.prototype.join.call(false) must return ""');
