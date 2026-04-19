// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
flags:
  - noStrict
description: |
  pending
esid: pending
---*/
// Test that functions in block that do not exhibit Annex B do not override
// previous functions that do exhibit Annex B.

function f() {
  var outerX;
  { function x() {1} outerX = x; }
  { { function x() {2}; } let x; }
  { let x; { function x() {3}; } }
  assert.sameValue(x, outerX);
}
f();

