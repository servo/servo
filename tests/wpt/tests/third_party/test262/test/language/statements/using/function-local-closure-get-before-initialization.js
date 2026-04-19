// Copyright (C) 2023 Ron Buckton. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-declarative-environment-records-getbindingvalue-n-s
description: >
    using: function local closure [[Get]] before initialization.
    (TDZ, Temporal Dead Zone)
features: [explicit-resource-management]
---*/

(function() {
  function f() { return x + 1; }

  assert.throws(ReferenceError, function() {
    f();
  });

  using x = null;
}());
