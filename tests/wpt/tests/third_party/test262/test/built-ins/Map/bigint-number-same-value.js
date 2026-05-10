// Copyright (C) 2021 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-map.prototype.set
description: >
  Observing the expected behavior of keys when a BigInt and Number have
  the same value.
info: |
  Map.prototype.set ( key , value )

  ...
  Let p be the Record {[[key]]: key, [[value]]: value}.
  Append p as the last element of entries.
  ...

features: [BigInt]
---*/

const number = 9007199254740991;
const bigint = 9007199254740991n;

const m = new Map([
  [number, number],
  [bigint, bigint],
]);

assert.sameValue(m.size, 2);
assert.sameValue(m.has(number), true);
assert.sameValue(m.has(bigint), true);

assert.sameValue(m.get(number), number);
assert.sameValue(m.get(bigint), bigint);

m.delete(number);
assert.sameValue(m.size, 1);
assert.sameValue(m.has(number), false);
m.delete(bigint);
assert.sameValue(m.size, 0);
assert.sameValue(m.has(bigint), false);

m.set(number, number);
assert.sameValue(m.size, 1);
m.set(bigint, bigint);
assert.sameValue(m.size, 2);
