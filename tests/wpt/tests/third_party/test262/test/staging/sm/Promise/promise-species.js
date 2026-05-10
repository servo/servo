// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  pending
esid: pending
---*/

assert.sameValue(Promise[Symbol.species], Promise);
let prop = Object.getOwnPropertyDescriptor(Promise, Symbol.species);
assert.sameValue('get' in prop, true);
assert.sameValue(typeof prop.get, 'function');
assert.sameValue('set' in prop, true);
assert.sameValue(prop.set, undefined);
