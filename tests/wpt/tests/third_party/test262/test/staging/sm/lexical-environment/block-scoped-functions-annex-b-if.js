// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
flags:
  - noStrict
description: |
  pending
esid: pending
---*/
var log = "";

function f(x) {
  if (x)
    function g() { return "g0"; }
  else
    function g() { return "g1"; }

  log += g();

  if (x)
    function g() { return "g2"; }
  else {
  }

  log += g();

  if (x) {
  } else
    function g() { return "g3"; }

  log += g();

  if (x)
    function g() { return "g4"; }

  log += g();
}

f(true);
f(false);

try {
  eval(`
    if (1)
      l: function foo() {}
  `);
} catch (e) {
  log += "e";
}

assert.sameValue(log, "g0g2g2g4g1g1g3g3e");
