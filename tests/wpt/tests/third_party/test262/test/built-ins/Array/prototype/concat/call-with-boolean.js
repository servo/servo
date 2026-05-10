// Copyright (c) 2020 Rick Waldron.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.concat
description: Array.prototype.concat applied to boolean primitive
---*/

assert.sameValue(
  Array.prototype.concat.call(true)[0] instanceof Boolean,
  true,
  'The result of evaluating (Array.prototype.concat.call(true)[0] instanceof Boolean) is expected to be true'
);
assert.sameValue(
  Array.prototype.concat.call(false)[0] instanceof Boolean,
  true,
  'The result of evaluating (Array.prototype.concat.call(false)[0] instanceof Boolean) is expected to be true'
);
