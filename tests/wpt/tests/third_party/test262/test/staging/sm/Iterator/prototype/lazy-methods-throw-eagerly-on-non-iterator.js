// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: pending
description: |
  Lazy %Iterator.prototype% methods don't throw when called with non-objects.
info: |
  Iterator Helpers proposal 1.1.1
features:
  - iterator-helpers
---*/

//
//
const methods = [
  iter => Iterator.prototype.map.bind(iter, x => x),
  iter => Iterator.prototype.filter.bind(iter, x => x),
  iter => Iterator.prototype.take.bind(iter, 1),
  iter => Iterator.prototype.drop.bind(iter, 0),
  iter => Iterator.prototype.flatMap.bind(iter, x => [x]),
];

for (const method of methods) {
  assert.throws(TypeError, method(undefined));
  assert.throws(TypeError, method(null));
  assert.throws(TypeError, method(0));
  assert.throws(TypeError, method(false));
  assert.throws(TypeError, method(''));
  assert.throws(TypeError, method(Symbol('')));

  // No error here.
  method({});
  method([]);
}

