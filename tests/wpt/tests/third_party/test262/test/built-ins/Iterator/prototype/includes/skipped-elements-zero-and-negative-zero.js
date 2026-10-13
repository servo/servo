// Copyright (C) 2026 Michael Ficarra. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iterator.prototype.includes
description: >
  skippedElements of 0 and -0 starts searching from the beginning
features: [iterator-includes]
---*/

assert.sameValue([4, 5, 6, 7].values().includes(8, 0), false);
assert.sameValue([4, 5, 6, 7].values().includes(7, 0), true);
assert.sameValue([4, 5, 6, 7].values().includes(6, 0), true);
assert.sameValue([4, 5, 6, 7].values().includes(5, 0), true);
assert.sameValue([4, 5, 6, 7].values().includes(4, 0), true);
assert.sameValue([4, 5, 6, 7].values().includes(3, 0), false);

assert.sameValue([4, 5, 6, 7].values().includes(8, -0), false);
assert.sameValue([4, 5, 6, 7].values().includes(7, -0), true);
assert.sameValue([4, 5, 6, 7].values().includes(6, -0), true);
assert.sameValue([4, 5, 6, 7].values().includes(5, -0), true);
assert.sameValue([4, 5, 6, 7].values().includes(4, -0), true);
assert.sameValue([4, 5, 6, 7].values().includes(3, -0), false);
