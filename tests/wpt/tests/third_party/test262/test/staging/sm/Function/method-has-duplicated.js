// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  pending
esid: pending
---*/
// Make sure duplicated name is allowed in non-strict.
function f0(a) {
}

// SyntaxError should be thrown if method definition has duplicated name.
assert.throws(SyntaxError, () => eval(`
({
  m1(a, a) {
  }
});
`));
assert.throws(SyntaxError, () => eval(`
({
  m2(a, ...a) {
  }
});
`));

