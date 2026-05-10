// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  pending
esid: pending
---*/
// Returning zero from the sort comparator...
let ta = new Int32Array([0, 1]).sort(() => 0);
assert.sameValue(ta[0], 0);
assert.sameValue(ta[1], 1);

// ... should give the same result as returning NaN.
let tb = new Int32Array([0, 1]).sort(() => NaN);
assert.sameValue(tb[0], 0);
assert.sameValue(tb[1], 1);

