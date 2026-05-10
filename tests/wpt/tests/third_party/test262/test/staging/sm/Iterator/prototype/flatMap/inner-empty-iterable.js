// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: pending
description: |
  %Iterator.prototype%.flatMap skips empty inner iterables.
info: |
  Iterator Helpers proposal 2.1.5.7 1. Repeat,
    ...
    i. Repeat, while innerAlive is true,
      ...
      iii. Let innerComplete be IteratorComplete(innerNext).
      ...
      v. If innerComplete is true, set innerAlive to false.
features:
  - iterator-helpers
---*/
let iter = [0, 1, 2, 3].values().flatMap(x => x % 2 ? [] : [x]);

for (const expected of [0, 2]) {
  const result = iter.next();
  assert.sameValue(result.value, expected);
  assert.sameValue(result.done, false);
}

let result = iter.next();
assert.sameValue(result.value, undefined);
assert.sameValue(result.done, true);

iter = [0, 1, 2, 3].values().flatMap(x => []);
result = iter.next();
assert.sameValue(result.value, undefined);
assert.sameValue(result.done, true);

