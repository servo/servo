// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  pending
esid: pending
---*/
// Scoping in the head of for(let;;) statements.

let x = 0;
for (let i = 0, a = () => i; i < 4; i++) {
  assert.sameValue(i, x++);
  assert.sameValue(a(), 0);
}
assert.sameValue(x, 4);

x = 11;
let q = 0;
for (let {[++q]: r} = [0, 11, 22], s = () => r; r < 13; r++) {
  assert.sameValue(r, x++);
  assert.sameValue(s(), 11);
}
assert.sameValue(x, 13);
assert.sameValue(q, 1);

