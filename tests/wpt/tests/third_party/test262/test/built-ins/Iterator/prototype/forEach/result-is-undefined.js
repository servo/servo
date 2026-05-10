// Copyright (C) 2023 Michael Ficarra. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iteratorprototype.forEach
description: >
  Iterator.prototype.forEach returns undefined
features: [iterator-helpers]
---*/
function* g() {}
let iter = g();
assert.sameValue(
  iter.forEach(() => {}),
  undefined
);
assert.sameValue(
  iter.forEach(() => 0),
  undefined
);
