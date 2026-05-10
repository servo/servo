// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: pending
description: |
  %Iterator.prototype%.map works even if the global Symbol has been clobbered..
features:
  - Symbol
  - Symbol.iterator
  - iterator-helpers
info: |
  Iterator is not enabled unconditionally
---*/
Symbol = undefined;
assert.throws(TypeError, () => Symbol.iterator);

const iterator = [0].values();
assert.sameValue(
  iterator.map(x => x + 1).next().value, 1,
  '`%Iterator.prototype%.map` still works after Symbol has been clobbered'
);

