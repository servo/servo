// Copyright (C) 2026 Michael Ficarra. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iterator.prototype.includes
description: >
  Omitted or undefined skippedElements behaves as 0
features: [iterator-includes]
---*/

assert.sameValue([4, 5, 6, 7].values().includes(4), true);
assert.sameValue([4, 5, 6, 7].values().includes(4, undefined), true);
assert.sameValue([4, 5, 6, 7].values().includes(4, 0), true);

assert.sameValue([4, 5, 6, 7].values().includes(8), false);
assert.sameValue([4, 5, 6, 7].values().includes(8, undefined), false);
assert.sameValue([4, 5, 6, 7].values().includes(8, 0), false);
