// Copyright (C) 2023 Kevin Gibbons. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-set.prototype.union
description: Set.prototype.union result ordering
features: [set-methods]
includes: [compareArray.js]
---*/

{
  const s1 = new Set([1, 2]);
  const s2 = new Set([2, 3]);

  assert.compareArray([...s1.union(s2)], [1, 2, 3]);
}

{
  const s1 = new Set([2, 3]);
  const s2 = new Set([1, 2]);

  assert.compareArray([...s1.union(s2)], [2, 3, 1]);
}

{
  const s1 = new Set([1, 2]);
  const s2 = new Set([3]);

  assert.compareArray([...s1.union(s2)], [1, 2, 3]);
}

{
  const s1 = new Set([3]);
  const s2 = new Set([1, 2]);

  assert.compareArray([...s1.union(s2)], [3, 1, 2]);
}
