// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  pending
esid: pending
---*/
// Test that parenthesized class expressions don't get their toString offsets
// messed up.

assert.sameValue((class {}).toString(), "class {}");
assert.sameValue(((class {})).toString(), "class {}");

