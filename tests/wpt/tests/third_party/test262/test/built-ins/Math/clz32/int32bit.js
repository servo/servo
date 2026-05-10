// Copyright (C) 2016 The V8 Project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-math.clz32
description: >
  Catches the int32bit value in the ToUint32 operation
info: |
  Math.clz32 ( x )

  1. Let n be ToUint32(x).
  2. Let p be the number of leading zero bits in the 32-bit binary representation of n.
  3. Return p.

  7.1.6 ToUint32 ( argument )

  [...]
  3. Let int be the mathematical value that is the same sign as number and whose
  magnitude is floor(abs(number)).
  4. Let int32bit be int modulo 232.
  5. Return int32bit.
  [...]
---*/

assert.sameValue(Math.clz32(4294967295), 0, "2**32-1");
assert.sameValue(Math.clz32(4294967296), 32, "2**32");
assert.sameValue(Math.clz32(4294967297), 31, "2**32+1");

assert.sameValue(Math.clz32(65535), 16, "2**16-1");
assert.sameValue(Math.clz32(65536), 15, "2**16");
assert.sameValue(Math.clz32(65537), 15, "2**16+1");

assert.sameValue(Math.clz32(255), 24, "2**8-1");
assert.sameValue(Math.clz32(256), 23, "2**8");
assert.sameValue(Math.clz32(257), 23, "2**8+1");

assert.sameValue(Math.clz32(-4294967295), 31, "-(2**32-1)");
assert.sameValue(Math.clz32(-4294967296), 32, "-(2**32)");
assert.sameValue(Math.clz32(-4294967297), 0, "-(2**32+1)");

assert.sameValue(Math.clz32(-65535), 0, "-(2**16-1)");
assert.sameValue(Math.clz32(-65536), 0, "-(2**16)");
assert.sameValue(Math.clz32(-65537), 0, "-(2**16+1)");

assert.sameValue(Math.clz32(-255), 0, "-(2**8-1)");
assert.sameValue(Math.clz32(-256), 0, "-(2**8)");
assert.sameValue(Math.clz32(-257), 0, "-(2**8+1)");
