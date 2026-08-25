// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
flags:
  - module
negative:
  phase: parse
  type: SyntaxError
description: |
  pending
esid: pending
---*/

async () => class { x = await 1 };
$DONOTEVALUATE();
