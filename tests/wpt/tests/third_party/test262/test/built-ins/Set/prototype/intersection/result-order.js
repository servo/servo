// Copyright (C) 2023 Kevin Gibbons. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-set.prototype.intersection
description: Set.prototype.intersection result ordering
features: [set-methods]
includes: [compareArray.js]
---*/

// when this.size ≤ other.size, results are ordered as in this
{
  const s1 = new Set([1, 3, 5]);
  const s2 = new Set([3, 2, 1]);

  assert.compareArray([...s1.intersection(s2)], [1, 3]);
}

{
  const s1 = new Set([3, 2, 1]);
  const s2 = new Set([1, 3, 5]);

  assert.compareArray([...s1.intersection(s2)], [3, 1]);
}

{
  const s1 = new Set([1, 3, 5]);
  const s2 = new Set([3, 2, 1, 0]);

  assert.compareArray([...s1.intersection(s2)], [1, 3]);
}

{
  const s1 = new Set([3, 2, 1]);
  const s2 = new Set([1, 3, 5, 7]);

  assert.compareArray([...s1.intersection(s2)], [3, 1]);
}


// when this.size > other.size, results are ordered as in other
{
  const s1 = new Set([3, 2, 1, 0]);
  const s2 = new Set([1, 3, 5]);

  assert.compareArray([...s1.intersection(s2)], [1, 3]);
}

{
  const s1 = new Set([1, 3, 5, 7]);
  const s2 = new Set([3, 2, 1]);

  assert.compareArray([...s1.intersection(s2)], [3, 1]);
}
