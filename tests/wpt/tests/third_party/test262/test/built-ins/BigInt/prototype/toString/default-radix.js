// Copyright 2018 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-bigint.prototype.tostring
description: toString with default radix
features: [BigInt]
---*/

assert.sameValue((-100n).toString(), "-100", "(-100n).toString() === '-100'");
assert.sameValue((0n).toString(), "0", "(0n).toString() === '0'");
assert.sameValue((100n).toString(), "100", "(100n).toString() === '100'");

assert.sameValue((-100n).toString(undefined), "-100",
                 "(-100n).toString(undefined) === '-100'");
assert.sameValue((0n).toString(undefined), "0",
                 "(0n).toString(undefined) === '0'");
assert.sameValue((100n).toString(undefined), "100",
                 "(100n).toString(undefined) === '100'");
