// Copyright (C) 2021 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-set.prototype.add
description: >
  Observing the expected behavior of keys when a BigInt and Number have
  the same value.
info: |
  Set.prototype.add ( value )

  ...
  For each element e of entries, do
    If e is not empty and SameValueZero(e, value) is true, then
    Return S.
  If value is -0, set value to +0.
  Append value as the last element of entries.
  ...

features: [BigInt]
---*/

const number = 9007199254740991;
const bigint = 9007199254740991n;

const s = new Set([
  number,
  bigint,
]);

assert.sameValue(s.size, 2);
assert.sameValue(s.has(number), true);
assert.sameValue(s.has(bigint), true);

s.delete(number);
assert.sameValue(s.size, 1);
assert.sameValue(s.has(number), false);
s.delete(bigint);
assert.sameValue(s.size, 0);
assert.sameValue(s.has(bigint), false);

s.add(number);
assert.sameValue(s.size, 1);
s.add(bigint);
assert.sameValue(s.size, 2);
