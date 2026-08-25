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

assert.throws(TypeError, () => iter.forEach());
assert.throws(TypeError, () => iter.forEach(undefined));
assert.throws(TypeError, () => iter.forEach(null));
assert.throws(TypeError, () => iter.forEach(0));
assert.throws(TypeError, () => iter.forEach(false));
assert.throws(TypeError, () => iter.forEach(''));
assert.throws(TypeError, () => iter.forEach(Symbol('')));
assert.throws(TypeError, () => iter.forEach({}));

