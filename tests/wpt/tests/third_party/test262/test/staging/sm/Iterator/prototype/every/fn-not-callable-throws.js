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

const iter = [].values();

assert.throws(TypeError, () => iter.every());
assert.throws(TypeError, () => iter.every(undefined));
assert.throws(TypeError, () => iter.every(null));
assert.throws(TypeError, () => iter.every(0));
assert.throws(TypeError, () => iter.every(false));
assert.throws(TypeError, () => iter.every(''));
assert.throws(TypeError, () => iter.every(Symbol('')));
assert.throws(TypeError, () => iter.every({}));

