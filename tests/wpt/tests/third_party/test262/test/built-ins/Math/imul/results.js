// Copyright (C) 2016 The V8 Project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-math.imul
description: >
  Return results
info: |
  Math.imul ( x, y )

  1. Let a be ToUint32(x).
  2. Let b be ToUint32(y).
  3. Let product be (a × b) modulo 232.
  4. If product ≥ 231, return product - 232; otherwise return product.
---*/

assert.sameValue(Math.imul(2, 4), 8, "(2, 4)");
assert.sameValue(Math.imul(-1, 8), -8, "(-1, 8)");
assert.sameValue(Math.imul(-2, -2), 4, "(-2, -2)");
assert.sameValue(Math.imul(0xffffffff, 5), -5, "(0xffffffff, 5)");
assert.sameValue(Math.imul(0xfffffffe, 5), -10, "(0xfffffffe, 5)");

assert.sameValue(Math.imul(-0, 7), 0);
assert.sameValue(Math.imul(7, -0), 0);

assert.sameValue(Math.imul(0.1, 7), 0);
assert.sameValue(Math.imul(7, 0.1), 0);
assert.sameValue(Math.imul(0.9, 7), 0);
assert.sameValue(Math.imul(7, 0.9), 0);
assert.sameValue(Math.imul(1.1, 7), 7);
assert.sameValue(Math.imul(7, 1.1), 7);
assert.sameValue(Math.imul(1.9, 7), 7);
assert.sameValue(Math.imul(7, 1.9), 7);

assert.sameValue(Math.imul(1073741824, 7), -1073741824);
assert.sameValue(Math.imul(7, 1073741824), -1073741824);
assert.sameValue(Math.imul(1073741824, 1073741824), 0);

assert.sameValue(Math.imul(-1073741824, 7), 1073741824);
assert.sameValue(Math.imul(7, -1073741824), 1073741824);
assert.sameValue(Math.imul(-1073741824, -1073741824), 0);

assert.sameValue(Math.imul(2147483648, 7), -2147483648);
assert.sameValue(Math.imul(7, 2147483648), -2147483648);
assert.sameValue(Math.imul(2147483648, 2147483648), 0);

assert.sameValue(Math.imul(-2147483648, 7), -2147483648);
assert.sameValue(Math.imul(7, -2147483648), -2147483648);
assert.sameValue(Math.imul(-2147483648, -2147483648), 0);

assert.sameValue(Math.imul(2147483647, 7), 2147483641);
assert.sameValue(Math.imul(7, 2147483647), 2147483641);
assert.sameValue(Math.imul(2147483647, 2147483647), 1);

assert.sameValue(Math.imul(65536, 65536), 0);
assert.sameValue(Math.imul(65535, 65536), -65536);
assert.sameValue(Math.imul(65536, 65535), -65536);
assert.sameValue(Math.imul(65535, 65535), -131071);
