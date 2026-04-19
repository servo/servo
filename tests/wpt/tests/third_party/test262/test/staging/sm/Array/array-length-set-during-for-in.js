// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  pending
esid: pending
---*/
var a = [0, 1];
var iterations = 0;
for (var k in a) {
  iterations++;
  a.length = 1;
}
assert.sameValue(iterations, 1);

