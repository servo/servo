// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    The [[Class]] property of the newly constructed object
    is set to "Date"
esid: sec-date-year-month-date-hours-minutes-seconds-ms
description: >
    Test based on delete prototype.toString - 2 arguments, (year,
    month)
---*/

var x1 = new Date(1899, 11);

assert.sameValue(
  Object.prototype.toString.call(x1),
  "[object Date]",
  'Object.prototype.toString.call(new Date(1899, 11)) must return "[object Date]"'
);

var x2 = new Date(1899, 12);

assert.sameValue(
  Object.prototype.toString.call(x2),
  "[object Date]",
  'Object.prototype.toString.call(new Date(1899, 12)) must return "[object Date]"'
);

var x3 = new Date(1900, 0);

assert.sameValue(
  Object.prototype.toString.call(x3),
  "[object Date]",
  'Object.prototype.toString.call(new Date(1900, 0)) must return "[object Date]"'
);

var x4 = new Date(1969, 11);

assert.sameValue(
  Object.prototype.toString.call(x4),
  "[object Date]",
  'Object.prototype.toString.call(new Date(1969, 11)) must return "[object Date]"'
);

var x5 = new Date(1969, 12);

assert.sameValue(
  Object.prototype.toString.call(x5),
  "[object Date]",
  'Object.prototype.toString.call(new Date(1969, 12)) must return "[object Date]"'
);

var x6 = new Date(1970, 0);

assert.sameValue(
  Object.prototype.toString.call(x6),
  "[object Date]",
  'Object.prototype.toString.call(new Date(1970, 0)) must return "[object Date]"'
);

var x7 = new Date(1999, 11);

assert.sameValue(
  Object.prototype.toString.call(x7),
  "[object Date]",
  'Object.prototype.toString.call(new Date(1999, 11)) must return "[object Date]"'
);

var x8 = new Date(1999, 12);

assert.sameValue(
  Object.prototype.toString.call(x8),
  "[object Date]",
  'Object.prototype.toString.call(new Date(1999, 12)) must return "[object Date]"'
);

var x9 = new Date(2000, 0);

assert.sameValue(
  Object.prototype.toString.call(x9),
  "[object Date]",
  'Object.prototype.toString.call(new Date(2000, 0)) must return "[object Date]"'
);

var x10 = new Date(2099, 11);

assert.sameValue(
  Object.prototype.toString.call(x10),
  "[object Date]",
  'Object.prototype.toString.call(new Date(2099, 11)) must return "[object Date]"'
);

var x11 = new Date(2099, 12);

assert.sameValue(
  Object.prototype.toString.call(x11),
  "[object Date]",
  'Object.prototype.toString.call(new Date(2099, 12)) must return "[object Date]"'
);

var x12 = new Date(2100, 0);

assert.sameValue(
  Object.prototype.toString.call(x12),
  "[object Date]",
  'Object.prototype.toString.call(new Date(2100, 0)) must return "[object Date]"'
);
