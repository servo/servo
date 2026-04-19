// Copyright (C) 2023 Michael Ficarra. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iteratorprototype.some
description: >
  Iterator.prototype.some returns a boolean
features: [iterator-helpers]
---*/
function* g() {}
let iter = g();
assert.sameValue(typeof iter.some(() => {}), 'boolean');
