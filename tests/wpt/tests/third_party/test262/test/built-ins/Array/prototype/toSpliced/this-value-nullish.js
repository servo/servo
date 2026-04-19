// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.tospliced
description: >
  Array.prototype.toSpliced throws if the receiver is null or undefined
info: |
  Array.prototype.toSpliced ( start, deleteCount, ...items )

  1. Let O be ? ToObject(this value).
  ...
features: [change-array-by-copy]
---*/

assert.throws(TypeError, () => {
  Array.prototype.toSpliced.call(null, 0, 0);
}, '`Array.prototype.toSpliced.call(null)` throws TypeError');

assert.throws(TypeError, () => {
  Array.prototype.toSpliced.call(undefined, 0, 0);
}, '`Array.prototype.toSpliced.call(undefined)` throws TypeError');
