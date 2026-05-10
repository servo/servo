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

const sum = (x, y) => x + y;
assert.throws(TypeError, Iterator.prototype.reduce.bind(undefined, sum));
assert.throws(TypeError, Iterator.prototype.reduce.bind({}, sum));
assert.throws(TypeError, Iterator.prototype.reduce.bind({next: 0}, sum));

