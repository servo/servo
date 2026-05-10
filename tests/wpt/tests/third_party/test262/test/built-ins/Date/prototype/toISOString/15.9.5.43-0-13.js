// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-date.prototype.toisostring
description: >
    Date.prototype.toISOString - RangeError is thrown when value of
    date is outside the valid range of time.
---*/

// As specified in ES5 15.9.1.14, time > 8.64e15 is not in the valid range.
var date = new Date(8.64e15 + 1);
assert.throws(RangeError, function() {
  date.toISOString();
});
