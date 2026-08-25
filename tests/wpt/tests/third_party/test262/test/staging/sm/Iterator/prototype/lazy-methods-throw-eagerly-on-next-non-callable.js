// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: pending
description: |
  Lazy %Iterator.prototype% methods don't throw when `next` is non-callable.
info: |
  Iterator Helpers proposal 1.1.1
features:
  - iterator-helpers
---*/

//
//
const methods = [
  next => Iterator.prototype.map.bind({next}, x => x),
  next => Iterator.prototype.filter.bind({next}, x => x),
  next => Iterator.prototype.take.bind({next}, 1),
  next => Iterator.prototype.drop.bind({next}, 0),
  next => Iterator.prototype.flatMap.bind({next}, x => [x]),
];

for (const method of methods) {
  method(0);
  method(false);
  method(undefined);
  method(null);
  method('');
  method(Symbol(''));
  method({});
  method([]);
}

