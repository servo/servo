// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
flags:
  - noStrict
description: |
  pending
esid: pending
---*/
function f() {}
var g = new Function();
delete Function;
function h() {}

assert.sameValue(f.__proto__, g.__proto__);
assert.sameValue(g.__proto__, h.__proto__);
assert.sameValue(false, "Function" in this);

assert.sameValue("ok", "ok", "bug 569306");
