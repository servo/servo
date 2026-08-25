// Copyright (C) 2026 Michael Ficarra. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iterator.prototype.includes
description: >
  The underlying iterator may be unable to be closed (has no return method)
features: [iterator-includes]
---*/

let iterator = [1, 2, 3, 4, 5].values();

assert.sameValue(iterator.return, undefined);
assert.sameValue(iterator.includes(4), true);

let step = iterator.next();
assert.sameValue(step.done, false);
assert.sameValue(step.value, 5);

step = iterator.next();
assert.sameValue(step.done, true);
assert.sameValue(step.value, undefined);
