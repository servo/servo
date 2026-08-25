// Copyright (C) 2018 Caio Lima. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: Relational comparison of BigInt and boolean values
esid: sec-abstract-relational-comparison
features: [BigInt]
---*/
assert.sameValue(0n < false, false, 'The result of (0n < false) is false');
assert.sameValue(false < 0n, false, 'The result of (false < 0n) is false');
assert.sameValue(0n < true, true, 'The result of (0n < true) is true');
assert.sameValue(true < 0n, false, 'The result of (true < 0n) is false');
assert.sameValue(1n < false, false, 'The result of (1n < false) is false');
assert.sameValue(false < 1n, true, 'The result of (false < 1n) is true');
assert.sameValue(1n < true, false, 'The result of (1n < true) is false');
assert.sameValue(true < 1n, false, 'The result of (true < 1n) is false');
assert.sameValue(31n < true, false, 'The result of (31n < true) is false');
assert.sameValue(true < 31n, true, 'The result of (true < 31n) is true');
assert.sameValue(-3n < true, true, 'The result of (-3n < true) is true');
assert.sameValue(true < -3n, false, 'The result of (true < -3n) is false');
assert.sameValue(-3n < false, true, 'The result of (-3n < false) is true');
assert.sameValue(false < -3n, false, 'The result of (false < -3n) is false');
