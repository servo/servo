// Copyright (C) 2026 Michael Ficarra. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iterator.prototype.includes
description: >
  Iterator.prototype.includes is callable
features: [iterator-includes]
---*/

function* g() {
  yield 0;
}

Iterator.prototype.includes.call(g(), 0);

g().includes(0);
