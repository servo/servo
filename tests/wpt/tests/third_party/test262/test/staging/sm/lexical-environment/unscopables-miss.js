// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
flags:
  - noStrict
description: |
  pending
esid: pending
---*/
// Trying to access a binding that doesn't exist due to @@unscopables
// is a ReferenceError.

with ({x: 1, [Symbol.unscopables]: {x: true}})
    assert.throws(ReferenceError, () => x);

