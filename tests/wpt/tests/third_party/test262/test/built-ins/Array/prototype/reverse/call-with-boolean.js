// Copyright (c) 2020 Rick Waldron.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.reverse
description: Array.prototype.reverse applied to boolean primitive
---*/

assert.sameValue(
  Array.prototype.reverse.call(true) instanceof Boolean,
  true,
  'The result of `(Array.prototype.reverse.call(true) instanceof Boolean)` is true'
);
assert.sameValue(
  Array.prototype.reverse.call(false) instanceof Boolean,
  true,
  'The result of `(Array.prototype.reverse.call(false) instanceof Boolean)` is true'
);
