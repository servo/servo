// Copyright (C) 2023 Ron Buckton. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-declarative-environment-records-getbindingvalue-n-s
description: >
    await using: block local closure [[Get]] before initialization.
    (TDZ, Temporal Dead Zone)
flags: [async]
includes: [asyncHelpers.js]
features: [explicit-resource-management]
---*/

asyncTest(async function () {
  function f() { return x + 1; }

  assert.throws(ReferenceError, function() {
    f();
  });

  await using x = null;
});
