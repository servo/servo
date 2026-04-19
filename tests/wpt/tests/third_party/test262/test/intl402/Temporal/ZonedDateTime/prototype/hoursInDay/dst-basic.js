// Copyright (C) 2018 Bloomberg LP. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-get-temporal.zoneddatetime.prototype.hoursinday
description: Basic tests around DST
features: [Temporal]
---*/

var hourBeforeDstStart = new Temporal.PlainDateTime(2000, 4, 2, 1).toZonedDateTime("America/Vancouver");
var dayBeforeDstStart = new Temporal.PlainDateTime(2000, 4, 1, 2, 30).toZonedDateTime("America/Vancouver");

// hoursInDay works with DST start
assert.sameValue(hourBeforeDstStart.hoursInDay, 23,
  "23 hours in dst start day");

// hoursInDay works with non-DST days
assert.sameValue(dayBeforeDstStart.hoursInDay, 24,
  "24 hours in a non-DST day");

// hoursInDay works with DST end
var dstEnd = Temporal.PlainDateTime.from("2000-10-29T01:00").toZonedDateTime("America/Vancouver");
assert.sameValue(dstEnd.hoursInDay, 25,
  "25 hours in DST end day");

var dayAfterSamoaDateLineChange = Temporal.PlainDateTime.from("2011-12-31T22:00").toZonedDateTime("Pacific/Apia");
var dayBeforeSamoaDateLineChange = Temporal.PlainDateTime.from("2011-12-29T22:00").toZonedDateTime("Pacific/Apia");

// hoursInDay works after Samoa date line change
assert.sameValue(dayAfterSamoaDateLineChange.hoursInDay, 24,
  "24 hours in day after Samoa date line change");

// hoursInDay works before Samoa date line change
assert.sameValue(dayBeforeSamoaDateLineChange.hoursInDay, 24,
  "24 hours in day before Samoa date line change");
