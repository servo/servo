// Copyright (C) 2024 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.hoursinday
description: >
  Basic tests for hoursInDay.
features: [Temporal]
---*/

var nsPerDay = 24n * 60n * 60n * 1000n * 1000n * 1000n;

var epochNanoseconds = [
  0n,
  nsPerDay,
  -nsPerDay,
];

var timeZones = [
  "UTC",
  "+00",
  "+01",
  "-01",
];

for (var timeZone of timeZones) {
  for (var epochNs of epochNanoseconds) {
    var zdt = new Temporal.ZonedDateTime(epochNs, timeZone);
    assert.sameValue(zdt.hoursInDay, 24, `epochNs = ${epochNs}, timeZone = ${timeZone}`);
  }
}
