// Copyright (C) 2017 Josh Wolfe. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-bigint.asuintn
description: BigInt.asUintN arithmetic test cases
info: |
  BigInt.asUintN ( bits, bigint )

  3. Return a BigInt representing bigint modulo 2**bits.

features: [BigInt]
---*/

assert.sameValue(BigInt.asUintN(0, -2n), 0n);
assert.sameValue(BigInt.asUintN(0, -1n), 0n);
assert.sameValue(BigInt.asUintN(0, 0n), 0n);
assert.sameValue(BigInt.asUintN(0, 1n), 0n);
assert.sameValue(BigInt.asUintN(0, 2n), 0n);

assert.sameValue(BigInt.asUintN(1, -3n), 1n);
assert.sameValue(BigInt.asUintN(1, -2n), 0n);
assert.sameValue(BigInt.asUintN(1, -1n), 1n);
assert.sameValue(BigInt.asUintN(1, 0n), 0n);
assert.sameValue(BigInt.asUintN(1, 1n), 1n);
assert.sameValue(BigInt.asUintN(1, 2n), 0n);
assert.sameValue(BigInt.asUintN(1, 3n), 1n);
assert.sameValue(BigInt.asUintN(1, -123456789012345678901n), 1n);
assert.sameValue(BigInt.asUintN(1, -123456789012345678900n), 0n);
assert.sameValue(BigInt.asUintN(1, 123456789012345678900n), 0n);
assert.sameValue(BigInt.asUintN(1, 123456789012345678901n), 1n);

assert.sameValue(BigInt.asUintN(2, -3n), 1n);
assert.sameValue(BigInt.asUintN(2, -2n), 2n);
assert.sameValue(BigInt.asUintN(2, -1n), 3n);
assert.sameValue(BigInt.asUintN(2, 0n), 0n);
assert.sameValue(BigInt.asUintN(2, 1n), 1n);
assert.sameValue(BigInt.asUintN(2, 2n), 2n);
assert.sameValue(BigInt.asUintN(2, 3n), 3n);
assert.sameValue(BigInt.asUintN(2, -123456789012345678901n), 3n);
assert.sameValue(BigInt.asUintN(2, -123456789012345678900n), 0n);
assert.sameValue(BigInt.asUintN(2, 123456789012345678900n), 0n);
assert.sameValue(BigInt.asUintN(2, 123456789012345678901n), 1n);

assert.sameValue(BigInt.asUintN(8, 0xabn), 0xabn);
assert.sameValue(BigInt.asUintN(8, 0xabcdn), 0xcdn);
assert.sameValue(BigInt.asUintN(8, 0xabcdef01n), 0x01n);
assert.sameValue(BigInt.asUintN(8, 0xabcdef0123456789abcdef0123n), 0x23n);
assert.sameValue(BigInt.asUintN(8, 0xabcdef0123456789abcdef0183n), 0x83n);

assert.sameValue(BigInt.asUintN(64, 0xabcdef0123456789abcdefn), 0x0123456789abcdefn);
assert.sameValue(BigInt.asUintN(65, 0xabcdef0123456789abcdefn), 0x10123456789abcdefn);

assert.sameValue(BigInt.asUintN(200,
    0xbffffffffffffffffffffffffffffffffffffffffffffffffffn),
  0x0ffffffffffffffffffffffffffffffffffffffffffffffffffn
);
assert.sameValue(BigInt.asUintN(201,
    0xbffffffffffffffffffffffffffffffffffffffffffffffffffn),
  0x1ffffffffffffffffffffffffffffffffffffffffffffffffffn
);

assert.sameValue(BigInt.asUintN(200,
    0xb89e081df68b65fedb32cffea660e55df9605650a603ad5fc54n),
  0x089e081df68b65fedb32cffea660e55df9605650a603ad5fc54n
);
assert.sameValue(BigInt.asUintN(201,
    0xb89e081df68b65fedb32cffea660e55df9605650a603ad5fc54n),
  0x189e081df68b65fedb32cffea660e55df9605650a603ad5fc54n
);
