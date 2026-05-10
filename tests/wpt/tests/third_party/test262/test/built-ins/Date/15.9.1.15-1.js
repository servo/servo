// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-date-time-string-format
description: >
    Date Time String Format - specified default values will be set for
    all optional fields(MM, DD, mm, ss and time zone) when they are
    absent
---*/

var result = false;
var expectedDateTimeStr = "1970-01-01T00:00:00.000Z";
var dateObj = new Date("1970");
var dateStr = dateObj.toISOString();
result = dateStr === expectedDateTimeStr;

assert(result, 'result !== true');
