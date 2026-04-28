// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    The [[Class]] property of the newly constructed object
    is set to "Date"
esid: sec-date-year-month-date-hours-minutes-seconds-ms
description: >
    Test based on overwriting prototype.toString - 7 arguments, (year,
    month, date, hours, minutes, seconds, ms)
---*/

Date.prototype.toString = Object.prototype.toString;

var x1 = new Date(1899, 11, 31, 23, 59, 59, 999);
assert.sameValue(x1.toString(), "[object Date]", 'x1.toString() must return "[object Date]"');

var x2 = new Date(1899, 12, 1, 0, 0, 0, 0);
assert.sameValue(x2.toString(), "[object Date]", 'x2.toString() must return "[object Date]"');

var x3 = new Date(1900, 0, 1, 0, 0, 0, 0);
assert.sameValue(x3.toString(), "[object Date]", 'x3.toString() must return "[object Date]"');

var x4 = new Date(1969, 11, 31, 23, 59, 59, 999);
assert.sameValue(x4.toString(), "[object Date]", 'x4.toString() must return "[object Date]"');

var x5 = new Date(1969, 12, 1, 0, 0, 0, 0);
assert.sameValue(x5.toString(), "[object Date]", 'x5.toString() must return "[object Date]"');

var x6 = new Date(1970, 0, 1, 0, 0, 0, 0);
assert.sameValue(x6.toString(), "[object Date]", 'x6.toString() must return "[object Date]"');

var x7 = new Date(1999, 11, 31, 23, 59, 59, 999);
assert.sameValue(x7.toString(), "[object Date]", 'x7.toString() must return "[object Date]"');

var x8 = new Date(1999, 12, 1, 0, 0, 0, 0);
assert.sameValue(x8.toString(), "[object Date]", 'x8.toString() must return "[object Date]"');

var x9 = new Date(2000, 0, 1, 0, 0, 0, 0);
assert.sameValue(x9.toString(), "[object Date]", 'x9.toString() must return "[object Date]"');

var x10 = new Date(2099, 11, 31, 23, 59, 59, 999);
assert.sameValue(x10.toString(), "[object Date]", 'x10.toString() must return "[object Date]"');

var x11 = new Date(2099, 12, 1, 0, 0, 0, 0);
assert.sameValue(x11.toString(), "[object Date]", 'x11.toString() must return "[object Date]"');

var x12 = new Date(2100, 0, 1, 0, 0, 0, 0);
assert.sameValue(x12.toString(), "[object Date]", 'x12.toString() must return "[object Date]"');
