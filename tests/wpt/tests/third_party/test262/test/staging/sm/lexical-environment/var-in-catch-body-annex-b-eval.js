// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
flags:
  - noStrict
description: |
  pending
esid: pending
---*/
// Tests annex B.3.5 that introduces a var via direct eval.

var x = "global-x";
var log = "";

// Tests that direct eval works.
function g() {
  try { throw 8; } catch (x) {
    eval("var x = 42;");
    log += x;
  }
  x = "g";
  log += x;
}
g();

assert.sameValue(x, "global-x");
assert.sameValue(log, "42g");

