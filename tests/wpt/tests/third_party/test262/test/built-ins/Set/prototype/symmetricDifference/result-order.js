// Copyright (C) 2023 Kevin Gibbons. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-set.prototype.symmetricdifference
description: Set.prototype.symmetricDifference result ordering
features: [set-methods]
includes: [compareArray.js]
---*/

// results are ordered as in this, then as in other
{
  const s1 = new Set([1, 2, 3, 4]);
  const s2 = new Set([6, 5, 4, 3]);

  assert.compareArray([...s1.symmetricDifference(s2)], [1, 2, 6, 5]);
}

{
  const s1 = new Set([6, 5, 4, 3]);
  const s2 = new Set([1, 2, 3, 4]);

  assert.compareArray([...s1.symmetricDifference(s2)], [6, 5, 1, 2]);
}
