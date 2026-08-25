// Copyright (C) 2017 Josh Wolfe. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-bigint.asintn
description: BigInt.asIntN arithmetic test cases
info: |
  BigInt.asIntN ( bits, bigint )

  3. Let mod be a BigInt representing bigint modulo 2**bits.
  4. If mod ≥ 2**bits - 1, return mod - 2**bits; otherwise, return mod.

features: [BigInt]
---*/

assert.sameValue(BigInt.asIntN(0, -2n), 0n);
assert.sameValue(BigInt.asIntN(0, -1n), 0n);
assert.sameValue(BigInt.asIntN(0, 0n), 0n);
assert.sameValue(BigInt.asIntN(0, 1n), 0n);
assert.sameValue(BigInt.asIntN(0, 2n), 0n);

assert.sameValue(BigInt.asIntN(1, -3n), -1n);
assert.sameValue(BigInt.asIntN(1, -2n), 0n);
assert.sameValue(BigInt.asIntN(1, -1n), -1n);
assert.sameValue(BigInt.asIntN(1, 0n), 0n);
assert.sameValue(BigInt.asIntN(1, 1n), -1n);
assert.sameValue(BigInt.asIntN(1, 2n), 0n);
assert.sameValue(BigInt.asIntN(1, 3n), -1n);
assert.sameValue(BigInt.asIntN(1, -123456789012345678901n), -1n);
assert.sameValue(BigInt.asIntN(1, -123456789012345678900n), 0n);
assert.sameValue(BigInt.asIntN(1, 123456789012345678900n), 0n);
assert.sameValue(BigInt.asIntN(1, 123456789012345678901n), -1n);

assert.sameValue(BigInt.asIntN(2, -3n), 1n);
assert.sameValue(BigInt.asIntN(2, -2n), -2n);
assert.sameValue(BigInt.asIntN(2, -1n), -1n);
assert.sameValue(BigInt.asIntN(2, 0n), 0n);
assert.sameValue(BigInt.asIntN(2, 1n), 1n);
assert.sameValue(BigInt.asIntN(2, 2n), -2n);
assert.sameValue(BigInt.asIntN(2, 3n), -1n);
assert.sameValue(BigInt.asIntN(2, -123456789012345678901n), -1n);
assert.sameValue(BigInt.asIntN(2, -123456789012345678900n), 0n);
assert.sameValue(BigInt.asIntN(2, 123456789012345678900n), 0n);
assert.sameValue(BigInt.asIntN(2, 123456789012345678901n), 1n);

assert.sameValue(BigInt.asIntN(8, 0xabn), -0x55n);
assert.sameValue(BigInt.asIntN(8, 0xabcdn), -0x33n);
assert.sameValue(BigInt.asIntN(8, 0xabcdef01n), 0x01n);
assert.sameValue(BigInt.asIntN(8, 0xabcdef0123456789abcdef0123n), 0x23n);
assert.sameValue(BigInt.asIntN(8, 0xabcdef0123456789abcdef0183n), -0x7dn);

assert.sameValue(BigInt.asIntN(64, 0xabcdef0123456789abcdefn), 0x0123456789abcdefn);
assert.sameValue(BigInt.asIntN(65, 0xabcdef0123456789abcdefn), -0xfedcba9876543211n);

assert.sameValue(BigInt.asIntN(200,
  0xcffffffffffffffffffffffffffffffffffffffffffffffffffn), -0x00000000000000000000000000000000000000000000000001n);
assert.sameValue(BigInt.asIntN(201,
    0xcffffffffffffffffffffffffffffffffffffffffffffffffffn),
  0xffffffffffffffffffffffffffffffffffffffffffffffffffn
);

assert.sameValue(BigInt.asIntN(200,
  0xc89e081df68b65fedb32cffea660e55df9605650a603ad5fc54n), -0x761f7e209749a0124cd3001599f1aa2069fa9af59fc52a03acn);
assert.sameValue(BigInt.asIntN(201,
    0xc89e081df68b65fedb32cffea660e55df9605650a603ad5fc54n),
  0x89e081df68b65fedb32cffea660e55df9605650a603ad5fc54n
);
