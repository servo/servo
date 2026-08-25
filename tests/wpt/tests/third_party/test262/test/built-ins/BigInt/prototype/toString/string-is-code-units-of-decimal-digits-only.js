// Copyright 2017 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-tostring-applied-to-the-bigint-type
description: BigInt .toString() returns only decimal digits, does not include BigIntLiteralSuffix
info: |
  ToString Applied to the BigInt Type

    The abstract operation ToString converts a BigInt i to String format as follows:

    ...
    Return the String consisting of the code units of the digits of the decimal representation of i.

features: [BigInt]
---*/

assert.sameValue(BigInt(0).toString(), "0", "BigInt(0).toString() === '0'");
assert.sameValue(BigInt(0n).toString(), "0", "BigInt(0n).toString() === '0'");
assert.sameValue(0n.toString(), "0", "0n.toString() === '0'");
