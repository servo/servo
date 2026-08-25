// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    When Date is called as part of a new expression it is
    a constructor: it initializes the newly created object
esid: sec-date-year-month-date-hours-minutes-seconds-ms
description: 5 arguments, (year, month, date, hours, minutes)
---*/
assert.sameValue(
  typeof new Date(1899, 11, 31, 23, 59),
  "object",
  'The value of `typeof new Date(1899, 11, 31, 23, 59)` is expected to be "object"'
);

assert.notSameValue(
  new Date(1899, 11, 31, 23, 59),
  undefined,
  'new Date(1899, 11, 31, 23, 59) is expected to not equal ``undefined``'
);

var x13 = new Date(1899, 11, 31, 23, 59);
assert.sameValue(typeof x13, "object", 'The value of `typeof x13` is expected to be "object"');

var x14 = new Date(1899, 11, 31, 23, 59);
assert.notSameValue(x14, undefined, 'The value of x14 is expected to not equal ``undefined``');

assert.sameValue(
  typeof new Date(1899, 12, 1, 0, 0),
  "object",
  'The value of `typeof new Date(1899, 12, 1, 0, 0)` is expected to be "object"'
);

assert.notSameValue(
  new Date(1899, 12, 1, 0, 0),
  undefined,
  'new Date(1899, 12, 1, 0, 0) is expected to not equal ``undefined``'
);

var x23 = new Date(1899, 12, 1, 0, 0);
assert.sameValue(typeof x23, "object", 'The value of `typeof x23` is expected to be "object"');

var x24 = new Date(1899, 12, 1, 0, 0);
assert.notSameValue(x24, undefined, 'The value of x24 is expected to not equal ``undefined``');

assert.sameValue(
  typeof new Date(1900, 0, 1, 0, 0),
  "object",
  'The value of `typeof new Date(1900, 0, 1, 0, 0)` is expected to be "object"'
);

assert.notSameValue(
  new Date(1900, 0, 1, 0, 0),
  undefined,
  'new Date(1900, 0, 1, 0, 0) is expected to not equal ``undefined``'
);

var x33 = new Date(1900, 0, 1, 0, 0);
assert.sameValue(typeof x33, "object", 'The value of `typeof x33` is expected to be "object"');

var x34 = new Date(1900, 0, 1, 0, 0);
assert.notSameValue(x34, undefined, 'The value of x34 is expected to not equal ``undefined``');

assert.sameValue(
  typeof new Date(1969, 11, 31, 23, 59),
  "object",
  'The value of `typeof new Date(1969, 11, 31, 23, 59)` is expected to be "object"'
);

assert.notSameValue(
  new Date(1969, 11, 31, 23, 59),
  undefined,
  'new Date(1969, 11, 31, 23, 59) is expected to not equal ``undefined``'
);

var x43 = new Date(1969, 11, 31, 23, 59);
assert.sameValue(typeof x43, "object", 'The value of `typeof x43` is expected to be "object"');

var x44 = new Date(1969, 11, 31, 23, 59);
assert.notSameValue(x44, undefined, 'The value of x44 is expected to not equal ``undefined``');

assert.sameValue(
  typeof new Date(1969, 12, 1, 0, 0),
  "object",
  'The value of `typeof new Date(1969, 12, 1, 0, 0)` is expected to be "object"'
);

assert.notSameValue(
  new Date(1969, 12, 1, 0, 0),
  undefined,
  'new Date(1969, 12, 1, 0, 0) is expected to not equal ``undefined``'
);

var x53 = new Date(1969, 12, 1, 0, 0);
assert.sameValue(typeof x53, "object", 'The value of `typeof x53` is expected to be "object"');

var x54 = new Date(1969, 12, 1, 0, 0);
assert.notSameValue(x54, undefined, 'The value of x54 is expected to not equal ``undefined``');

assert.sameValue(
  typeof new Date(1970, 0, 1, 0, 0),
  "object",
  'The value of `typeof new Date(1970, 0, 1, 0, 0)` is expected to be "object"'
);

assert.notSameValue(
  new Date(1970, 0, 1, 0, 0),
  undefined,
  'new Date(1970, 0, 1, 0, 0) is expected to not equal ``undefined``'
);

var x63 = new Date(1970, 0, 1, 0, 0);
assert.sameValue(typeof x63, "object", 'The value of `typeof x63` is expected to be "object"');

var x64 = new Date(1970, 0, 1, 0, 0);
assert.notSameValue(x64, undefined, 'The value of x64 is expected to not equal ``undefined``');

assert.sameValue(
  typeof new Date(1999, 11, 31, 23, 59),
  "object",
  'The value of `typeof new Date(1999, 11, 31, 23, 59)` is expected to be "object"'
);

assert.notSameValue(
  new Date(1999, 11, 31, 23, 59),
  undefined,
  'new Date(1999, 11, 31, 23, 59) is expected to not equal ``undefined``'
);

var x73 = new Date(1999, 11, 31, 23, 59);
assert.sameValue(typeof x73, "object", 'The value of `typeof x73` is expected to be "object"');

var x74 = new Date(1999, 11, 31, 23, 59);
assert.notSameValue(x74, undefined, 'The value of x74 is expected to not equal ``undefined``');

assert.sameValue(
  typeof new Date(1999, 12, 1, 0, 0),
  "object",
  'The value of `typeof new Date(1999, 12, 1, 0, 0)` is expected to be "object"'
);

assert.notSameValue(
  new Date(1999, 12, 1, 0, 0),
  undefined,
  'new Date(1999, 12, 1, 0, 0) is expected to not equal ``undefined``'
);

var x83 = new Date(1999, 12, 1, 0, 0);
assert.sameValue(typeof x83, "object", 'The value of `typeof x83` is expected to be "object"');

var x84 = new Date(1999, 12, 1, 0, 0);
assert.notSameValue(x84, undefined, 'The value of x84 is expected to not equal ``undefined``');

assert.sameValue(
  typeof new Date(2000, 0, 1, 0, 0),
  "object",
  'The value of `typeof new Date(2000, 0, 1, 0, 0)` is expected to be "object"'
);

assert.notSameValue(
  new Date(2000, 0, 1, 0, 0),
  undefined,
  'new Date(2000, 0, 1, 0, 0) is expected to not equal ``undefined``'
);

var x93 = new Date(2000, 0, 1, 0, 0);
assert.sameValue(typeof x93, "object", 'The value of `typeof x93` is expected to be "object"');

var x94 = new Date(2000, 0, 1, 0, 0);
assert.notSameValue(x94, undefined, 'The value of x94 is expected to not equal ``undefined``');

assert.sameValue(
  typeof new Date(2099, 11, 31, 23, 59),
  "object",
  'The value of `typeof new Date(2099, 11, 31, 23, 59)` is expected to be "object"'
);

assert.notSameValue(
  new Date(2099, 11, 31, 23, 59),
  undefined,
  'new Date(2099, 11, 31, 23, 59) is expected to not equal ``undefined``'
);

var x103 = new Date(2099, 11, 31, 23, 59);
assert.sameValue(typeof x103, "object", 'The value of `typeof x103` is expected to be "object"');

var x104 = new Date(2099, 11, 31, 23, 59);
assert.notSameValue(x104, undefined, 'The value of x104 is expected to not equal ``undefined``');

assert.sameValue(
  typeof new Date(2099, 12, 1, 0, 0),
  "object",
  'The value of `typeof new Date(2099, 12, 1, 0, 0)` is expected to be "object"'
);

assert.notSameValue(
  new Date(2099, 12, 1, 0, 0),
  undefined,
  'new Date(2099, 12, 1, 0, 0) is expected to not equal ``undefined``'
);

var x113 = new Date(2099, 12, 1, 0, 0);
assert.sameValue(typeof x113, "object", 'The value of `typeof x113` is expected to be "object"');

var x114 = new Date(2099, 12, 1, 0, 0);
assert.notSameValue(x114, undefined, 'The value of x114 is expected to not equal ``undefined``');

assert.sameValue(
  typeof new Date(2100, 0, 1, 0, 0),
  "object",
  'The value of `typeof new Date(2100, 0, 1, 0, 0)` is expected to be "object"'
);

assert.notSameValue(
  new Date(2100, 0, 1, 0, 0),
  undefined,
  'new Date(2100, 0, 1, 0, 0) is expected to not equal ``undefined``'
);

var x123 = new Date(2100, 0, 1, 0, 0);
assert.sameValue(typeof x123, "object", 'The value of `typeof x123` is expected to be "object"');

var x124 = new Date(2100, 0, 1, 0, 0);
assert.notSameValue(x124, undefined, 'The value of x124 is expected to not equal ``undefined``');
