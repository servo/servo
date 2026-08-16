// Copyright (C) 2026 Michael Ficarra. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iterator.prototype.includes
description: >
  Basic positive and negative matches
features: [iterator-includes]
---*/

let arr = [3, 6, 9];

assert.sameValue(arr.values().includes(0), false);
assert.sameValue(arr.values().includes(1), false);
assert.sameValue(arr.values().includes(2), false);
assert.sameValue(arr.values().includes(3), true);
assert.sameValue(arr.values().includes(4), false);
assert.sameValue(arr.values().includes(5), false);
assert.sameValue(arr.values().includes(6), true);
assert.sameValue(arr.values().includes(7), false);
assert.sameValue(arr.values().includes(8), false);
assert.sameValue(arr.values().includes(9), true);
assert.sameValue(arr.values().includes(10), false);
