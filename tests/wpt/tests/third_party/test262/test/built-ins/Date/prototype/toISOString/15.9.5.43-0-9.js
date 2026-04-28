// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-date.prototype.toisostring
description: >
    Date.prototype.toISOString - RangeError is not thrown when value
    of date is Date(1970, 0, -99999999, 0, 0, 0, 0), the time zone is
    UTC(0)
---*/

var timeZoneMinutes = new Date(0).getTimezoneOffset() * (-1);
var date, dateStr;

if (timeZoneMinutes > 0) {
  date = new Date(1970, 0, -99999999, 0, 0, 0, 0);

  assert.throws(RangeError, function() {
    date.toISOString();
  });
} else {
  date = new Date(1970, 0, -99999999, 0, 0 + timeZoneMinutes + 60, 0, 0);

  dateStr = date.toISOString();

  assert.sameValue(dateStr[dateStr.length - 1], "Z");
}
