// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  pending
esid: pending
---*/
// Array with trailing hole as explicit "magic elements hole".
assert.sameValue([,].includes(), true);
assert.sameValue([,].includes(undefined), true);
assert.sameValue([,].includes(undefined, 0), true);
assert.sameValue([,].includes(null), false);
assert.sameValue([,].includes(null, 0), false);

// Array with trailing hole with no explicit "magic elements hole".
assert.sameValue(Array(1).includes(), true);
assert.sameValue(Array(1).includes(undefined), true);
assert.sameValue(Array(1).includes(undefined, 0), true);
assert.sameValue(Array(1).includes(null), false);
assert.sameValue(Array(1).includes(null, 0), false);

