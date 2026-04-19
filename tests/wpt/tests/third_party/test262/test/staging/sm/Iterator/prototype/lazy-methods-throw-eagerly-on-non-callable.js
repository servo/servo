// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: pending
description: |
  Lazy %Iterator.prototype% methods throw eagerly when passed non-callables.
info: |
  Iterator Helpers proposal 2.1.5
features:
  - iterator-helpers
---*/

//
//
const methods = [
  (iter, fn) => iter.map(fn),
  (iter, fn) => iter.filter(fn),
  (iter, fn) => iter.flatMap(fn),
];

for (const method of methods) {
  assert.throws(TypeError, () => method(Iterator.prototype, 0));
  assert.throws(TypeError, () => method(Iterator.prototype, false));
  assert.throws(TypeError, () => method(Iterator.prototype, undefined));
  assert.throws(TypeError, () => method(Iterator.prototype, null));
  assert.throws(TypeError, () => method(Iterator.prototype, ''));
  assert.throws(TypeError, () => method(Iterator.prototype, Symbol('')));
  assert.throws(TypeError, () => method(Iterator.prototype, {}));

  assert.throws(TypeError, () => method([].values(), 0));
  assert.throws(TypeError, () => method([].values(), false));
  assert.throws(TypeError, () => method([].values(), undefined));
  assert.throws(TypeError, () => method([].values(), null));
  assert.throws(TypeError, () => method([].values(), ''));
  assert.throws(TypeError, () => method([].values(), Symbol('')));
  assert.throws(TypeError, () => method([].values(), {}));
}

