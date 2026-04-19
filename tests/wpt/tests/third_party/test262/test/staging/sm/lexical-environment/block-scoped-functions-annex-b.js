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

log += typeof f;

{
  log += f();

  function f() {
    return "f1";
  }
}

log += f();

function g() {
  log += typeof h;

  {
    log += h();

    function h() {
      return "h1";
    }
  }

  log += h();
}

g();

assert.sameValue(log, "undefinedf1f1undefinedh1h1");
