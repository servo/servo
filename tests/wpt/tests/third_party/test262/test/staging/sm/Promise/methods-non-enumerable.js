// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  pending
esid: pending
---*/

assert.sameValue(Object.keys(Promise).length, 0);
assert.sameValue(Object.keys(Promise.prototype).length, 0);
