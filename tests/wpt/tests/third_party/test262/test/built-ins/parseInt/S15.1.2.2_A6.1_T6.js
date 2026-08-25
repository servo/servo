// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    If S contains any character that is not a radix-R digit,
    then let Z be the substring of S consisting of all characters before
    the first such character; otherwise, let Z be S
esid: sec-parseint-string-radix
description: Complex test. Radix-R notation in [0..9]
---*/

assert.sameValue(parseInt("0123456789", 2), 1, 'parseInt("0123456789", 2) must return 1');
assert.sameValue(parseInt("01234567890", 3), 5, 'parseInt("01234567890", 3) must return 5');
assert.sameValue(parseInt("01234567890", 4), 27, 'parseInt("01234567890", 4) must return 27');
assert.sameValue(parseInt("01234567890", 5), 194, 'parseInt("01234567890", 5) must return 194');
assert.sameValue(parseInt("01234567890", 6), 1865, 'parseInt("01234567890", 6) must return 1865');
assert.sameValue(parseInt("01234567890", 7), 22875, 'parseInt("01234567890", 7) must return 22875');
assert.sameValue(parseInt("01234567890", 8), 342391, 'parseInt("01234567890", 8) must return 342391');
assert.sameValue(parseInt("01234567890", 9), 6053444, 'parseInt("01234567890", 9) must return 6053444');

assert.sameValue(
  parseInt("01234567890", 10),
  Number(1234567890),
  'parseInt("01234567890", 10) must return the same value returned by Number(1234567890)'
);
