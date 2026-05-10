// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
flags:
  - noStrict
description: |
  pending
esid: pending
---*/
// Test that var declarations of arguments "shadows" the arguments binding
// used in parameter expressions.

function g8(h = () => arguments) {
  var arguments = 0;
  assert.sameValue(arguments, 0);
  assert.sameValue(arguments === h(), false);
}
g8();

function g9(h = () => arguments) {
  var arguments;
  assert.sameValue(void 0 === arguments, false);
  assert.sameValue(h(), arguments);
  arguments = 0;
  assert.sameValue(arguments, 0);
  assert.sameValue(arguments === h(), false);
}
g9();

