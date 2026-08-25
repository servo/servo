// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
features:
  - iterator-helpers
info: |
  Iterator is not enabled unconditionally
description: |
  pending
esid: pending
---*/

const iter = [1, 2, 3].values();
const log = [];
const fn = (value) => {
  log.push(value.toString());
  return value % 2 == 1;
};

assert.sameValue(iter.every(fn), false);
assert.sameValue(log.join(','), '1,2');

