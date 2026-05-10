// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
flags:
  - onlyStrict
description: |
  pending
esid: pending
---*/

var log = "";

function f() {
  return "f0";
}

log += f();

{
  log += f();

  function f() {
    return "f1";
  }

  log += f();
}

log += f();

function g() {
  function h() {
    return "h0";
  }

  log += h();

  {
    log += h();

    function h() {
      return "h1";
    }

    log += h();
  }

  log += h();
}

g();

assert.sameValue(log, "f0f1f1f0h0h1h1h0");
