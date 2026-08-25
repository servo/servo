// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.equals
description: ZonedDateTimes constructed from equivalent parameters are equal.
features: [Temporal]
---*/

const zdt = new Temporal.ZonedDateTime(0n, "-05:00", "iso8601");

// constructed from equivalent parameters are equal
const zdt2 = Temporal.ZonedDateTime.from({
  year: 1969,
  month: 12,
  day: 31,
  hour: 19,
  timeZone: "-05:00",
  calendar: "iso8601",
});
assert(zdt.equals(zdt2));
assert(zdt2.equals(zdt));
