// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    When Date is called as part of a new expression it is
    a constructor: it initialises the newly created object
esid: sec-date-value
description: Checking types of newly created objects and it values
includes: [dateConstants.js]
---*/
assert.sameValue(
  typeof new Date(date_1899_end),
  "object",
  'The value of `typeof new Date(date_1899_end)` is expected to be "object"'
);

assert.notSameValue(new Date(date_1899_end), undefined, 'new Date(date_1899_end) is expected to not equal ``undefined``');

var x13 = new Date(date_1899_end);
assert.sameValue(typeof x13, "object", 'The value of `typeof x13` is expected to be "object"');

var x14 = new Date(date_1899_end);
assert.notSameValue(x14, undefined, 'The value of x14 is expected to not equal ``undefined``');

assert.sameValue(
  typeof new Date(date_1900_start),
  "object",
  'The value of `typeof new Date(date_1900_start)` is expected to be "object"'
);

assert.notSameValue(
  new Date(date_1900_start),
  undefined,
  'new Date(date_1900_start) is expected to not equal ``undefined``'
);

var x23 = new Date(date_1900_start);
assert.sameValue(typeof x23, "object", 'The value of `typeof x23` is expected to be "object"');

var x24 = new Date(date_1900_start);
assert.notSameValue(x24, undefined, 'The value of x24 is expected to not equal ``undefined``');

assert.sameValue(
  typeof new Date(date_1969_end),
  "object",
  'The value of `typeof new Date(date_1969_end)` is expected to be "object"'
);

assert.notSameValue(new Date(date_1969_end), undefined, 'new Date(date_1969_end) is expected to not equal ``undefined``');

var x33 = new Date(date_1969_end);
assert.sameValue(typeof x33, "object", 'The value of `typeof x33` is expected to be "object"');

var x34 = new Date(date_1969_end);
assert.notSameValue(x34, undefined, 'The value of x34 is expected to not equal ``undefined``');

assert.sameValue(
  typeof new Date(date_1970_start),
  "object",
  'The value of `typeof new Date(date_1970_start)` is expected to be "object"'
);

assert.notSameValue(
  new Date(date_1970_start),
  undefined,
  'new Date(date_1970_start) is expected to not equal ``undefined``'
);

var x43 = new Date(date_1970_start);
assert.sameValue(typeof x43, "object", 'The value of `typeof x43` is expected to be "object"');

var x44 = new Date(date_1970_start);
assert.notSameValue(x44, undefined, 'The value of x44 is expected to not equal ``undefined``');

assert.sameValue(
  typeof new Date(date_1999_end),
  "object",
  'The value of `typeof new Date(date_1999_end)` is expected to be "object"'
);

assert.notSameValue(new Date(date_1999_end), undefined, 'new Date(date_1999_end) is expected to not equal ``undefined``');

var x53 = new Date(date_1999_end);
assert.sameValue(typeof x53, "object", 'The value of `typeof x53` is expected to be "object"');

var x54 = new Date(date_1999_end);
assert.notSameValue(x54, undefined, 'The value of x54 is expected to not equal ``undefined``');

assert.sameValue(
  typeof new Date(date_2000_start),
  "object",
  'The value of `typeof new Date(date_2000_start)` is expected to be "object"'
);

assert.notSameValue(
  new Date(date_2000_start),
  undefined,
  'new Date(date_2000_start) is expected to not equal ``undefined``'
);

var x63 = new Date(date_2000_start);
assert.sameValue(typeof x63, "object", 'The value of `typeof x63` is expected to be "object"');

var x64 = new Date(date_2000_start);
assert.notSameValue(x64, undefined, 'The value of x64 is expected to not equal ``undefined``');

assert.sameValue(
  typeof new Date(date_2099_end),
  "object",
  'The value of `typeof new Date(date_2099_end)` is expected to be "object"'
);

assert.notSameValue(new Date(date_2099_end), undefined, 'new Date(date_2099_end) is expected to not equal ``undefined``');

var x73 = new Date(date_2099_end);
assert.sameValue(typeof x73, "object", 'The value of `typeof x73` is expected to be "object"');

var x74 = new Date(date_2099_end);
assert.notSameValue(x74, undefined, 'The value of x74 is expected to not equal ``undefined``');

assert.sameValue(
  typeof new Date(date_2100_start),
  "object",
  'The value of `typeof new Date(date_2100_start)` is expected to be "object"'
);

assert.notSameValue(
  new Date(date_2100_start),
  undefined,
  'new Date(date_2100_start) is expected to not equal ``undefined``'
);

var x83 = new Date(date_2100_start);
assert.sameValue(typeof x83, "object", 'The value of `typeof x83` is expected to be "object"');

var x84 = new Date(date_2100_start);
assert.notSameValue(x84, undefined, 'The value of x84 is expected to not equal ``undefined``');
