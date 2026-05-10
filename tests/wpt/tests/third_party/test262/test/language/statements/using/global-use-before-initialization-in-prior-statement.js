// Copyright (C) 2023 Ron Buckton. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-declarative-environment-records-getbindingvalue-n-s
description: >
    using: global use before initialization in prior statement.
    (TDZ, Temporal Dead Zone)
negative:
  phase: runtime
  type: ReferenceError
features: [explicit-resource-management]
---*/

{
  x; using x = null;
}
