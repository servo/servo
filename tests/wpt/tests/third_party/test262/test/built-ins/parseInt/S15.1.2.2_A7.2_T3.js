// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    Compute the mathematical integer value
    that is represented by Z in radix-R notation, using the
    letters A-Z and a-z for digits with values 10 through 35.
    Compute the number value for Result(16)
esid: sec-parseint-string-radix
description: Checking algorithm for R = 16
---*/

assert.sameValue(parseInt("0x1", 16), 1, 'parseInt("0x1", 16) must return 1');
assert.sameValue(parseInt("0X10", 16), 16, 'parseInt("0X10", 16) must return 16');
assert.sameValue(parseInt("0x100", 16), 256, 'parseInt("0x100", 16) must return 256');
assert.sameValue(parseInt("0X1000", 16), 4096, 'parseInt("0X1000", 16) must return 4096');
assert.sameValue(parseInt("0x10000", 16), 65536, 'parseInt("0x10000", 16) must return 65536');
assert.sameValue(parseInt("0X100000", 16), 1048576, 'parseInt("0X100000", 16) must return 1048576');
assert.sameValue(parseInt("0x1000000", 16), 16777216, 'parseInt("0x1000000", 16) must return 16777216');
assert.sameValue(parseInt("0x10000000", 16), 268435456, 'parseInt("0x10000000", 16) must return 268435456');
assert.sameValue(parseInt("0x100000000", 16), 4294967296, 'parseInt("0x100000000", 16) must return 4294967296');
assert.sameValue(parseInt("0x1000000000", 16), 68719476736, 'parseInt("0x1000000000", 16) must return 68719476736');
assert.sameValue(parseInt("0x10000000000", 16), 1099511627776, 'parseInt("0x10000000000", 16) must return 1099511627776');

assert.sameValue(
  parseInt("0x100000000000", 16),
  17592186044416,
  'parseInt("0x100000000000", 16) must return 17592186044416'
);

assert.sameValue(
  parseInt("0x1000000000000", 16),
  281474976710656,
  'parseInt("0x1000000000000", 16) must return 281474976710656'
);

assert.sameValue(
  parseInt("0x10000000000000", 16),
  4503599627370496,
  'parseInt("0x10000000000000", 16) must return 4503599627370496'
);

assert.sameValue(
  parseInt("0x100000000000000", 16),
  72057594037927936,
  'parseInt("0x100000000000000", 16) must return 72057594037927936'
);

assert.sameValue(
  parseInt("0x1000000000000000", 16),
  1152921504606846976,
  'parseInt("0x1000000000000000", 16) must return 1152921504606846976'
);

assert.sameValue(
  parseInt("0x10000000000000000", 16),
  18446744073709551616,
  'parseInt("0x10000000000000000", 16) must return 18446744073709551616'
);

assert.sameValue(
  parseInt("0x100000000000000000", 16),
  295147905179352825856,
  'parseInt("0x100000000000000000", 16) must return 295147905179352825856'
);

assert.sameValue(
  parseInt("0x1000000000000000000", 16),
  4722366482869645213696,
  'parseInt("0x1000000000000000000", 16) must return 4722366482869645213696'
);

assert.sameValue(
  parseInt("0x10000000000000000000", 16),
  75557863725914323419136,
  'parseInt("0x10000000000000000000", 16) must return 75557863725914323419136'
);
