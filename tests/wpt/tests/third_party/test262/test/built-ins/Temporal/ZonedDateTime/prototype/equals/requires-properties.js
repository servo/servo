// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.equals
description: At least the required properties must be present.
features: [Temporal]
---*/

const zdt = new Temporal.ZonedDateTime(0n, "-05:00", "iso8601");

// at least the required properties must be present
assert(!zdt.equals({
  year: 1969,
  month: 12,
  day: 31,
  timeZone: "-05:00"
}));
assert.throws(TypeError, () => zdt.equals({
  month: 12,
  day: 31,
  timeZone: "-05:00"
}));
assert.throws(TypeError, () => zdt.equals({
  year: 1969,
  day: 31,
  timeZone: "-05:00"
}));
assert.throws(TypeError, () => zdt.equals({
  year: 1969,
  month: 12,
  timeZone: "-05:00"
}));
assert.throws(TypeError, () => zdt.equals({
  year: 1969,
  month: 12,
  day: 31
}));
assert.throws(TypeError, () => zdt.equals({
  years: 1969,
  months: 12,
  days: 31,
  timeZone: "-05:00",
  calendarName: "iso8601"
}));
