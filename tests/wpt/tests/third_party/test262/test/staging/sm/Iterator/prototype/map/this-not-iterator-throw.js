// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: pending
description: |
  Eagerly throw TypeError when `this` is not an iterator.
features:
  - iterator-helpers
---*/
//

const mapper = (x) => x;

assert.throws(TypeError, () => Iterator.prototype.map.call(undefined, mapper));
assert.throws(TypeError, () => Iterator.prototype.map.call(null, mapper));
assert.throws(TypeError, () => Iterator.prototype.map.call(0, mapper));
assert.throws(TypeError, () => Iterator.prototype.map.call(false, mapper));
assert.throws(TypeError, () => Iterator.prototype.map.call('', mapper));
assert.throws(TypeError, () => Iterator.prototype.map.call(new Symbol(''), mapper));

