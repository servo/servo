// Copyright (C) 2023 Michael Ficarra. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iteratorprototype.constructor
description: Property descriptor
info: |
  `Iterator.prototype.constructor` is an accessor property with attributes { [[Enumerable]]: *false*, [[Configurable]]: *true* }
features: [iterator-helpers]
---*/
let desc = Object.getOwnPropertyDescriptor(Iterator.prototype, 'constructor');
assert.sameValue(typeof desc.get, 'function');
assert.sameValue(typeof desc.set, 'function');
assert.sameValue(desc.configurable, true);
assert.sameValue(desc.enumerable, false);
assert.sameValue(desc.value, undefined);
assert.sameValue(desc.writable, undefined);
