// Copyright (C) 2026 Michael Ficarra. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iterator.prototype.includes
description: >
  Includes can find elements in an infinite iterator
features: [iterator-includes]
---*/

let gen = function* () {
  for (let i = 0; ; ++i) {
    yield i;
  }
};

assert.sameValue(gen().includes(1000), true);
