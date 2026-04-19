// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
flags:
  - noStrict
description: |
  pending
esid: pending
---*/
// Tests by André Bargull <andrebargull@googlemail.com>

// Annex B.3.3.1
function f1() {
    { function* g() {} }
    assert.sameValue(typeof g, "undefined");
}
f1();

// Annex B.3.3.2
{ function* g() {} }
assert.sameValue(typeof g, "undefined");

// Annex B.3.3.3
function f2() {
    eval("{ function* g() {} }");
    assert.sameValue(typeof g, "undefined");
}
f2();

// Annex B.3.3.3
eval("{ function* g() {} }");
assert.sameValue(typeof g, "undefined");

