// Copyright (C) 2018 Bloomberg LP. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.tostring
description: Basic tests for various combinations of the options calendar, timeZone, and offset.
features: [Temporal]
---*/

const zdt1 = Temporal.ZonedDateTime.from("1976-11-18T15:23+00:00[UTC]");
const zdt = zdt1.withCalendar("gregory");
assert.sameValue(zdt.toString({
  timeZoneName: "never",
  calendarName: "never"
}), "1976-11-18T15:23:00+00:00");
assert.sameValue(zdt.toString({
  offset: "never",
  calendarName: "never"
}), "1976-11-18T15:23:00[UTC]");
assert.sameValue(zdt.toString({
  offset: "never",
  timeZoneName: "never"
}), "1976-11-18T15:23:00[u-ca=gregory]");
assert.sameValue(zdt.toString({
  offset: "never",
  timeZoneName: "never",
  calendarName: "never"
}), "1976-11-18T15:23:00");
