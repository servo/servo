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

assert.throws(TypeError, Iterator.prototype.toArray.bind(undefined));
assert.throws(TypeError, Iterator.prototype.toArray.bind({}));
assert.throws(TypeError, Iterator.prototype.toArray.bind({next: 0}));

