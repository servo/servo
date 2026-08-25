// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.toreversed
description: >
  Array.prototype.toReversed throws if the receiver is null or undefined
info: |
  Array.prototype.toReversed ( )

  1. Let O be ? ToObject(this value).
  ...
features: [change-array-by-copy]
---*/

assert.throws(TypeError, () => {
  Array.prototype.toReversed.call(null);
});

assert.throws(TypeError, () => {
  Array.prototype.toReversed.call(undefined);
});
