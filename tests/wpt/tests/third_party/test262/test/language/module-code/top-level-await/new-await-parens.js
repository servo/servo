// Copyright (C) 2020 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: prod-AwaitExpression
description: >
  `new (await Constructor)` returns instance of Constructor
flags: [module, async]
features: [top-level-await]
---*/

assert.sameValue((new (await Number)).valueOf(), 0);
assert.sameValue((new (await String)).valueOf(), '');
assert.sameValue((new (await Boolean)).valueOf(), false);
assert.sameValue((new (await Array)).length, 0);
assert.sameValue((new (await Map)).size, 0);
assert.sameValue((new (await Set)).size, 0);

$DONE();
