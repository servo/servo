// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
flags:
  - noStrict
description: |
  pending
esid: pending
---*/
/* Check we can't delete a var-declared arguments in global space. */
var arguments = 42;
assert.sameValue(delete arguments, false, "arguments defined as global variable");
