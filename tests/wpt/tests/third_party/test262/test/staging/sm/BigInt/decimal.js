// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  pending
esid: pending
---*/

// Any copyright is dedicated to the Public Domain.
// https://creativecommons.org/licenses/publicdomain/

// Check base-10 BigInt string conversion
const decimalTests = [
    [32n, -1n, 1n, "4294967295"],
    [32n, -1n, -1n, "-4294967295"],
    [32n, 0n, 1n, "4294967296"],
    [32n, 0n, -1n, "-4294967296"],
    [32n, 1n, 1n, "4294967297"],
    [32n, 1n, -1n, "-4294967297"],
    [64n, -1n, 1n, "18446744073709551615"],
    [64n, -1n, -1n, "-18446744073709551615"],
    [64n, 0n, 1n, "18446744073709551616"],
    [64n, 0n, -1n, "-18446744073709551616"],
    [64n, 1n, 1n, "18446744073709551617"],
    [64n, 1n, -1n, "-18446744073709551617"],
    [128n, -1n, 1n, "340282366920938463463374607431768211455"],
    [128n, -1n, -1n, "-340282366920938463463374607431768211455"],
    [128n, 0n, 1n, "340282366920938463463374607431768211456"],
    [128n, 0n, -1n, "-340282366920938463463374607431768211456"],
    [128n, 1n, 1n, "340282366920938463463374607431768211457"],
    [128n, 1n, -1n, "-340282366920938463463374607431768211457"],
];
for (const [power, offset, sign, result] of decimalTests) {
    assert.sameValue(((2n**power+offset)*sign).toString(),
             result);
}
