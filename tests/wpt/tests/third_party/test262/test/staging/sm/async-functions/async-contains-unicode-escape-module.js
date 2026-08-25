// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  async/await containing escapes
esid: pending
negative:
  phase: parse
  type: SyntaxError
flags:
  - module
---*/

$DONOTEVALUATE();

export default \u0061sync function f() {}
