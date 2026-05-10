// Copyright (C) 2018 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: Relational comparison of BigInt and string values
esid: sec-abstract-relational-comparison
features: [BigInt]
---*/
assert.sameValue('0n' <= 1n, false, 'The result of ("0n" <= 1n) is false');
assert.sameValue('0.' <= 1n, false, 'The result of ("0." <= 1n) is false');
assert.sameValue('.0' <= 1n, false, 'The result of (".0" <= 1n) is false');
assert.sameValue('0/1' <= 1n, false, 'The result of ("0/1" <= 1n) is false');
assert.sameValue('z0' <= 1n, false, 'The result of ("z0" <= 1n) is false');
assert.sameValue('0z' <= 1n, false, 'The result of ("0z" <= 1n) is false');
assert.sameValue('++0' <= 1n, false, 'The result of ("++0" <= 1n) is false');
assert.sameValue('--0' <= 1n, false, 'The result of ("--0" <= 1n) is false');
assert.sameValue('0e0' <= 1n, false, 'The result of ("0e0" <= 1n) is false');
assert.sameValue('Infinity' <= 1n, false, 'The result of ("Infinity" <= 1n) is false');
assert.sameValue(0n <= '1n', false, 'The result of (0n <= "1n") is false');
assert.sameValue(0n <= '1.', false, 'The result of (0n <= "1.") is false');
assert.sameValue(0n <= '.1', false, 'The result of (0n <= ".1") is false');
assert.sameValue(0n <= '1/1', false, 'The result of (0n <= "1/1") is false');
assert.sameValue(0n <= 'z1', false, 'The result of (0n <= "z1") is false');
assert.sameValue(0n <= '1z', false, 'The result of (0n <= "1z") is false');
assert.sameValue(0n <= '++1', false, 'The result of (0n <= "++1") is false');
assert.sameValue(0n <= '--1', false, 'The result of (0n <= "--1") is false');
assert.sameValue(0n <= '1e0', false, 'The result of (0n <= "1e0") is false');
assert.sameValue(0n <= 'Infinity', false, 'The result of (0n <= "Infinity") is false');
