// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  pending
esid: pending
---*/
// Make sure that we can plumb new.target, even if the results are going to
// throw.

assert.throws(TypeError, () => new ""(...Array()));

assert.throws(TypeError, () => new ""());
assert.throws(TypeError, () => new ""(1));

