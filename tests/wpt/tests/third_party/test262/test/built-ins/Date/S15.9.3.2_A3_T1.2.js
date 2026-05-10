// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    The [[Class]] property of the newly constructed object
    is set to "Date"
esid: sec-date-value
description: Test based on overwriting prototype.toString
includes: [dateConstants.js]
---*/

Date.prototype.toString = Object.prototype.toString;

var x1 = new Date(date_1899_end);
assert.sameValue(x1.toString(), "[object Date]", 'x1.toString() must return "[object Date]"');

var x2 = new Date(date_1900_start);
assert.sameValue(x2.toString(), "[object Date]", 'x2.toString() must return "[object Date]"');

var x3 = new Date(date_1969_end);
assert.sameValue(x3.toString(), "[object Date]", 'x3.toString() must return "[object Date]"');

var x4 = new Date(date_1970_start);
assert.sameValue(x4.toString(), "[object Date]", 'x4.toString() must return "[object Date]"');

var x5 = new Date(date_1999_end);
assert.sameValue(x5.toString(), "[object Date]", 'x5.toString() must return "[object Date]"');

var x6 = new Date(date_2000_start);
assert.sameValue(x6.toString(), "[object Date]", 'x6.toString() must return "[object Date]"');

var x7 = new Date(date_2099_end);
assert.sameValue(x7.toString(), "[object Date]", 'x7.toString() must return "[object Date]"');

var x8 = new Date(date_2100_start);
assert.sameValue(x8.toString(), "[object Date]", 'x8.toString() must return "[object Date]"');
