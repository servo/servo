// Copyright (C) 2023 Ron Buckton. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-declarative-environment-records-getbindingvalue-n-s
description: >
    using: function local use before initialization in declaration statement.
    (TDZ, Temporal Dead Zone)
features: [explicit-resource-management]
---*/

assert.throws(ReferenceError, function() {
  (function() {
    using x = x + 1;
  }());
});
