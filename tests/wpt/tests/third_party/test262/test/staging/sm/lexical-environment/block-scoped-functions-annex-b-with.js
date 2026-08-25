// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
flags:
  - noStrict
description: |
  pending
esid: pending
---*/
var o = { f: "string-f" };
with (o) {
  var desc = Object.getOwnPropertyDescriptor(this, "f");
  assert.sameValue(desc.value, undefined);
  assert.sameValue(desc.writable, true);
  assert.sameValue(desc.enumerable, true);
  assert.sameValue(desc.configurable, false);
  function f() {
    return "fun-f";
  }
}

// Annex B explicitly assigns to the nearest VariableEnvironment, so the
// with-object "o" should have its property unchanged.
assert.sameValue(o.f, "string-f");
assert.sameValue(f(), "fun-f");

