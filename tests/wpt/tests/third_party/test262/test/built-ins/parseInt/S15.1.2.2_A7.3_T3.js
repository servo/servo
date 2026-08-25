// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Return sign * Result(17)
esid: sec-parseint-string-radix
description: Checking algorithm for R = 10
---*/

assert.sameValue(parseInt("-1", 10), -1, 'parseInt("-1", 10) must return -1');
assert.sameValue(parseInt("-10", 10), -10, 'parseInt("-10", 10) must return -10');
assert.sameValue(parseInt("-100", 10), -100, 'parseInt("-100", 10) must return -100');
assert.sameValue(parseInt("-1000", 10), -1000, 'parseInt("-1000", 10) must return -1000');
assert.sameValue(parseInt("-10000", 10), -10000, 'parseInt("-10000", 10) must return -10000');
assert.sameValue(parseInt("-100000", 10), -100000, 'parseInt("-100000", 10) must return -100000');
assert.sameValue(parseInt("-1000000", 10), -1000000, 'parseInt("-1000000", 10) must return -1000000');
assert.sameValue(parseInt("-10000000", 10), -10000000, 'parseInt("-10000000", 10) must return -10000000');
assert.sameValue(parseInt("-100000000", 10), -100000000, 'parseInt("-100000000", 10) must return -100000000');
assert.sameValue(parseInt("-1000000000", 10), -1000000000, 'parseInt("-1000000000", 10) must return -1000000000');
assert.sameValue(parseInt("-10000000000", 10), -10000000000, 'parseInt("-10000000000", 10) must return -10000000000');
assert.sameValue(parseInt("-100000000000", 10), -100000000000, 'parseInt("-100000000000", 10) must return -100000000000');

assert.sameValue(
  parseInt("-1000000000000", 10),
  -1000000000000,
  'parseInt("-1000000000000", 10) must return -1000000000000'
);

assert.sameValue(
  parseInt("-10000000000000", 10),
  -10000000000000,
  'parseInt("-10000000000000", 10) must return -10000000000000'
);

assert.sameValue(
  parseInt("-100000000000000", 10),
  -100000000000000,
  'parseInt("-100000000000000", 10) must return -100000000000000'
);

assert.sameValue(
  parseInt("-1000000000000000", 10),
  -1000000000000000,
  'parseInt("-1000000000000000", 10) must return -1000000000000000'
);

assert.sameValue(
  parseInt("-10000000000000000", 10),
  -10000000000000000,
  'parseInt("-10000000000000000", 10) must return -10000000000000000'
);

assert.sameValue(
  parseInt("-100000000000000000", 10),
  -100000000000000000,
  'parseInt("-100000000000000000", 10) must return -100000000000000000'
);

assert.sameValue(
  parseInt("-1000000000000000000", 10),
  -1000000000000000000,
  'parseInt("-1000000000000000000", 10) must return -1000000000000000000'
);

assert.sameValue(
  parseInt("-10000000000000000000", 10),
  -10000000000000000000,
  'parseInt("-10000000000000000000", 10) must return -10000000000000000000'
);
