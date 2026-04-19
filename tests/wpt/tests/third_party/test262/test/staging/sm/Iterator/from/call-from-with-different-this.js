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
  next() {
    assert.sameValue(arguments.length, 0);
    return {done: false, value: 0};
  },
};
const wrap = Iterator.from.call(undefined, iter);

const result = wrap.next("next argument is ignored");
assert.sameValue(result.done, false);
assert.sameValue(result.value, 0);

const returnResult = wrap.return("return argument is ignored");
assert.sameValue(returnResult.done, true);
assert.sameValue(returnResult.value, undefined);

