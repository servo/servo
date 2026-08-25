// Copyright (C) 2024 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-get-temporal.zoneddatetime.prototype.yearofweek
description: >
  Basic tests for yearOfWeek.
features: [Temporal]
---*/

var nsPerDay = 864n * 10n ** 11n;

var zdt;

zdt = new Temporal.ZonedDateTime(0n, "UTC");
assert.sameValue(zdt.yearOfWeek, 1970);

zdt = new Temporal.ZonedDateTime(-3n * nsPerDay, "UTC")
assert.sameValue(zdt.yearOfWeek, 1970);

zdt = new Temporal.ZonedDateTime(-4n * nsPerDay, "UTC")
assert.sameValue(zdt.yearOfWeek, 1969);

zdt = new Temporal.ZonedDateTime(367n * nsPerDay, "UTC")
assert.sameValue(zdt.yearOfWeek, 1970);

zdt = new Temporal.ZonedDateTime(368n * nsPerDay, "UTC")
assert.sameValue(zdt.yearOfWeek, 1971);
