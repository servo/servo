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
const iter = {
  next: () => ({ done: false, value: 0 }),
};

const wrap = Iterator.from(iter);

iter.next = () => ({ done: true, value: undefined });

let {done, value} = wrap.next();
assert.sameValue(done, false);
assert.sameValue(value, 0);

