// Copyright (C) 2023 Kevin Gibbons. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-set.prototype.difference
description: Set.prototype.difference orders results as in this, regardless of sizes
features: [set-methods]
includes: [compareArray.js]
---*/

{
  const s1 = new Set([1, 2, 3, 4]);
  const s2 = new Set([6, 5, 3, 2]);

  assert.compareArray([...s1.difference(s2)], [1, 4]);
}

{
  const s1 = new Set([6, 5, 3, 2]);
  const s2 = new Set([1, 2, 3, 4]);

  assert.compareArray([...s1.difference(s2)], [6, 5]);
}

{
  const s1 = new Set([1, 2, 3, 4]);
  const s2 = new Set([7, 6, 5, 3, 2]);

  assert.compareArray([...s1.difference(s2)], [1, 4]);
}

{
  const s1 = new Set([7, 6, 5, 3, 2]);
  const s2 = new Set([1, 2, 3, 4]);

  assert.compareArray([...s1.difference(s2)], [7, 6, 5]);
}

