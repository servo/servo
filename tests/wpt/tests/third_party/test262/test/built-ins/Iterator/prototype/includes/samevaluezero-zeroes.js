// Copyright (C) 2026 Michael Ficarra. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iterator.prototype.includes
description: >
  Includes uses SameValueZero for +0 and -0
features: [iterator-includes]
---*/

let positive = [+0];
let negative = [-0];

assert.sameValue(positive.values().includes(+0), true);
assert.sameValue(positive.values().includes(-0), true);
assert.sameValue(negative.values().includes(+0), true);
assert.sameValue(negative.values().includes(-0), true);
