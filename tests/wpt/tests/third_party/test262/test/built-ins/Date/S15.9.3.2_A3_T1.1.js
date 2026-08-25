// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    The [[Class]] property of the newly constructed object
    is set to "Date"
esid: sec-date-value
description: Test based on delete prototype.toString
includes: [dateConstants.js]
---*/

var x1 = new Date(date_1899_end);

assert.sameValue(
  Object.prototype.toString.call(x1),
  "[object Date]",
  'Object.prototype.toString.call(new Date(date_1899_end)) must return "[object Date]"'
);

var x2 = new Date(date_1900_start);

assert.sameValue(
  Object.prototype.toString.call(x2),
  "[object Date]",
  'Object.prototype.toString.call(new Date(date_1900_start)) must return "[object Date]"'
);

var x3 = new Date(date_1969_end);

assert.sameValue(
  Object.prototype.toString.call(x3),
  "[object Date]",
  'Object.prototype.toString.call(new Date(date_1969_end)) must return "[object Date]"'
);

var x4 = new Date(date_1970_start);

assert.sameValue(
  Object.prototype.toString.call(x4),
  "[object Date]",
  'Object.prototype.toString.call(new Date(date_1970_start)) must return "[object Date]"'
);

var x5 = new Date(date_1999_end);

assert.sameValue(
  Object.prototype.toString.call(x5),
  "[object Date]",
  'Object.prototype.toString.call(new Date(date_1999_end)) must return "[object Date]"'
);

var x6 = new Date(date_2000_start);

assert.sameValue(
  Object.prototype.toString.call(x6),
  "[object Date]",
  'Object.prototype.toString.call(new Date(date_2000_start)) must return "[object Date]"'
);

var x7 = new Date(date_2099_end);

assert.sameValue(
  Object.prototype.toString.call(x7),
  "[object Date]",
  'Object.prototype.toString.call(new Date(date_2099_end)) must return "[object Date]"'
);

var x8 = new Date(date_2100_start);

assert.sameValue(
  Object.prototype.toString.call(x8),
  "[object Date]",
  'Object.prototype.toString.call(new Date(date_2100_start)) must return "[object Date]"'
);
