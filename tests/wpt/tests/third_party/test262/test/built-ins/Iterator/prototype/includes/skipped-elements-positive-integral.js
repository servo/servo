// Copyright (C) 2026 Michael Ficarra. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iterator.prototype.includes
description: >
  Positive integral skippedElements skips that many iterated values
features: [iterator-includes]
---*/

assert.sameValue([4, 5, 6, 7].values().includes(4, 1), false);
assert.sameValue([4, 5, 6, 7].values().includes(4, 2), false);
assert.sameValue([4, 5, 6, 7].values().includes(4, 3), false);
assert.sameValue([4, 5, 6, 7].values().includes(4, 4), false);
assert.sameValue([4, 5, 6, 7].values().includes(4, 5), false);

assert.sameValue([4, 5, 6, 7].values().includes(5, 1), true);
assert.sameValue([4, 5, 6, 7].values().includes(5, 2), false);
assert.sameValue([4, 5, 6, 7].values().includes(5, 3), false);
assert.sameValue([4, 5, 6, 7].values().includes(5, 4), false);
assert.sameValue([4, 5, 6, 7].values().includes(5, 5), false);

assert.sameValue([4, 5, 6, 7].values().includes(6, 1), true);
assert.sameValue([4, 5, 6, 7].values().includes(6, 2), true);
assert.sameValue([4, 5, 6, 7].values().includes(6, 3), false);
assert.sameValue([4, 5, 6, 7].values().includes(6, 4), false);
assert.sameValue([4, 5, 6, 7].values().includes(6, 5), false);

assert.sameValue([4, 5, 6, 7].values().includes(7, 1), true);
assert.sameValue([4, 5, 6, 7].values().includes(7, 2), true);
assert.sameValue([4, 5, 6, 7].values().includes(7, 3), true);
assert.sameValue([4, 5, 6, 7].values().includes(7, 4), false);
assert.sameValue([4, 5, 6, 7].values().includes(7, 5), false);
