// Copyright (C) 2023 Anthony Frehner and Kevin Gibbons. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-set.prototype.difference
description: Set.prototype.difference doesn't work with arrays
features: [set-methods]
---*/

const s1 = new Set([1, 2]);
const s2 = [3];
assert.throws(
  TypeError,
  function () {
    s1.difference(s2);
  },
  "Throws an error when an array is used"
);
