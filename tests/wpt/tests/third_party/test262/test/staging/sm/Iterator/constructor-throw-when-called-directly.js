// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
  Iterator constructor throws when called directly.

  Iterator is not enabled unconditionally
features:
  - iterator-helpers
description: |
  pending
esid: pending
---*/
assert.throws(TypeError, () => new Iterator());

