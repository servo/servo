// Copyright (C) 2026 Michael Ficarra. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iterator.prototype.includes
description: >
  Iterator.prototype.includes returns a boolean
features: [iterator-includes]
---*/

function* g() {}

let iter = g();

assert.sameValue(typeof iter.includes(0), 'boolean');
