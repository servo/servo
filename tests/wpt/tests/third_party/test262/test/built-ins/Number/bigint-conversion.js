// Copyright (C) 2017 Robin Templeton. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: BigInt to Number conversion
esid: pending
features: [BigInt, exponentiation]
---*/

assert.sameValue(Number(0n), 0);
assert.sameValue(+(new Number(0n)), +(new Number(0)));

assert.sameValue(Number(2n**53n), 9007199254740992);
assert.sameValue(Number(2n**53n + 1n), 9007199254740992);
assert.sameValue(Number(2n**53n + 2n), 9007199254740994);
assert.sameValue(Number(2n**53n + 3n), 9007199254740996);
assert.sameValue(Number(2n**53n + 4n), 9007199254740996);

assert.sameValue(Number(-(2n**53n)), -9007199254740992);
assert.sameValue(Number(-(2n**53n + 1n)), -9007199254740992);
assert.sameValue(Number(-(2n**53n + 2n)), -9007199254740994);
assert.sameValue(Number(-(2n**53n + 3n)), -9007199254740996);
assert.sameValue(Number(-(2n**53n + 4n)), -9007199254740996);
