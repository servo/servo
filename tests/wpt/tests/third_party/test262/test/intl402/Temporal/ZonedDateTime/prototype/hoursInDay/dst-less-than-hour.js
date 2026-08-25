// Copyright (C) 2018 Bloomberg LP. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-get-temporal.zoneddatetime.prototype.hoursinday
description: Test TZDB edge cases where DST shift is less than one hour.
features: [Temporal]
---*/

// hoursInDay works with non-hour DST change
var zdt1 = Temporal.ZonedDateTime.from("2020-10-04T12:00[Australia/Lord_Howe]");
assert.sameValue(zdt1.hoursInDay, 23.5,
  "half hour less in day");
var zdt2 = Temporal.ZonedDateTime.from("2020-04-05T12:00[Australia/Lord_Howe]");
assert.sameValue(zdt2.hoursInDay, 24.5,
  "half hour more in day");

// hoursInDay works with non-half-hour DST change
var zdt = Temporal.ZonedDateTime.from("1933-01-01T12:00[Asia/Singapore]");
assert(Math.abs(zdt.hoursInDay - 23.666666666666668) < Number.EPSILON,
  "20 minutes less in day");
