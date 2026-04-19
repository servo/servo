// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
  Iterator.from throws when called with an object with a non-callable @@iterator property.

  Iterator is not enabled unconditionally
features:
  - iterator-helpers
description: |
  pending
esid: pending
---*/
assert.throws(TypeError, () => Iterator.from({ [Symbol.iterator]: 0 }));
assert.throws(TypeError, () => Iterator.from({ [Symbol.iterator]: false }));
assert.throws(TypeError, () => Iterator.from({ [Symbol.iterator]: "" }));
assert.throws(TypeError, () => Iterator.from({ [Symbol.iterator]: {} }));
assert.throws(TypeError, () => Iterator.from({ [Symbol.iterator]: Symbol('') }));

