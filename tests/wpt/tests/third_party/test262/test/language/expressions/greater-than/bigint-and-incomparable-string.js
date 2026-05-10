// Copyright (C) 2018 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: Relational comparison of BigInt and string values
esid: sec-abstract-relational-comparison
features: [BigInt]
---*/
assert.sameValue(1n > '0n', false, 'The result of (1n > "0n") is false');
assert.sameValue(1n > '0.', false, 'The result of (1n > "0.") is false');
assert.sameValue(1n > '.0', false, 'The result of (1n > ".0") is false');
assert.sameValue(1n > '0/1', false, 'The result of (1n > "0/1") is false');
assert.sameValue(1n > 'z0', false, 'The result of (1n > "z0") is false');
assert.sameValue(1n > '0z', false, 'The result of (1n > "0z") is false');
assert.sameValue(1n > '++0', false, 'The result of (1n > "++0") is false');
assert.sameValue(1n > '--0', false, 'The result of (1n > "--0") is false');
assert.sameValue(1n > '0e0', false, 'The result of (1n > "0e0") is false');
assert.sameValue(1n > 'Infinity', false, 'The result of (1n > "Infinity") is false');
assert.sameValue('1n' > 0n, false, 'The result of ("1n" > 0n) is false');
assert.sameValue('1.' > 0n, false, 'The result of ("1." > 0n) is false');
assert.sameValue('.1' > 0n, false, 'The result of (".1" > 0n) is false');
assert.sameValue('1/1' > 0n, false, 'The result of ("1/1" > 0n) is false');
assert.sameValue('z1' > 0n, false, 'The result of ("z1" > 0n) is false');
assert.sameValue('1z' > 0n, false, 'The result of ("1z" > 0n) is false');
assert.sameValue('++1' > 0n, false, 'The result of ("++1" > 0n) is false');
assert.sameValue('--1' > 0n, false, 'The result of ("--1" > 0n) is false');
assert.sameValue('1e0' > 0n, false, 'The result of ("1e0" > 0n) is false');
assert.sameValue('Infinity' > 0n, false, 'The result of ("Infinity" > 0n) is false');
