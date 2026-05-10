// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Return sign * Result(17)
esid: sec-parseint-string-radix
description: Checking algorithm for R = 2
---*/

assert.sameValue(parseInt("-1", 2), -1, 'parseInt("-1", 2) must return -1');
assert.sameValue(parseInt("-11", 2), -3, 'parseInt("-11", 2) must return -3');
assert.sameValue(parseInt("-111", 2), -7, 'parseInt("-111", 2) must return -7');
assert.sameValue(parseInt("-1111", 2), -15, 'parseInt("-1111", 2) must return -15');
assert.sameValue(parseInt("-11111", 2), -31, 'parseInt("-11111", 2) must return -31');
assert.sameValue(parseInt("-111111", 2), -63, 'parseInt("-111111", 2) must return -63');
assert.sameValue(parseInt("-1111111", 2), -127, 'parseInt("-1111111", 2) must return -127');
assert.sameValue(parseInt("-11111111", 2), -255, 'parseInt("-11111111", 2) must return -255');
assert.sameValue(parseInt("-111111111", 2), -511, 'parseInt("-111111111", 2) must return -511');
assert.sameValue(parseInt("-1111111111", 2), -1023, 'parseInt("-1111111111", 2) must return -1023');
assert.sameValue(parseInt("-11111111111", 2), -2047, 'parseInt("-11111111111", 2) must return -2047');
assert.sameValue(parseInt("-111111111111", 2), -4095, 'parseInt("-111111111111", 2) must return -4095');
assert.sameValue(parseInt("-1111111111111", 2), -8191, 'parseInt("-1111111111111", 2) must return -8191');
assert.sameValue(parseInt("-11111111111111", 2), -16383, 'parseInt("-11111111111111", 2) must return -16383');
assert.sameValue(parseInt("-111111111111111", 2), -32767, 'parseInt("-111111111111111", 2) must return -32767');
assert.sameValue(parseInt("-1111111111111111", 2), -65535, 'parseInt("-1111111111111111", 2) must return -65535');
assert.sameValue(parseInt("-11111111111111111", 2), -131071, 'parseInt("-11111111111111111", 2) must return -131071');
assert.sameValue(parseInt("-111111111111111111", 2), -262143, 'parseInt("-111111111111111111", 2) must return -262143');
assert.sameValue(parseInt("-1111111111111111111", 2), -524287, 'parseInt("-1111111111111111111", 2) must return -524287');

assert.sameValue(
  parseInt("-11111111111111111111", 2),
  -1048575,
  'parseInt("-11111111111111111111", 2) must return -1048575'
);
