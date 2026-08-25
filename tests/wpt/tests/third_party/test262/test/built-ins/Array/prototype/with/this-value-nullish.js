// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.with
description: >
  Array.prototype.with throws if the receiver is null or undefined
info: |
  Array.prototype.with ( index, value )

  1. Let O be ? ToObject(this value).
  ...
features: [change-array-by-copy]
---*/

assert.throws(TypeError, () => {
  Array.prototype.with.call(null, 0, 0);
});

assert.throws(TypeError, () => {
  Array.prototype.with.call(undefined, 0, 0);
});
