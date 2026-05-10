// Copyright (C) 2024 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.hoursinday
description: >
  GetStartOfDay throws a RangeError for values outside the valid limits.
info: |
  get Temporal.ZonedDateTime.prototype.hoursInDay

  ...
  7. Let todayNs be ? GetStartOfDay(timeZone, today).
  8. Let tomorrowNs be ? GetStartOfDay(timeZone, tomorrow).
  ...
features: [Temporal]
---*/

var zdt;

// GetStartOfDay for |today| fails.
zdt = new Temporal.ZonedDateTime(-864n * 10n**19n, "-01");
assert.throws(RangeError, () => zdt.hoursInDay);

// GetStartOfDay for |today| fails.
zdt = new Temporal.ZonedDateTime(-864n * 10n**19n, "+01");
assert.throws(RangeError, () => zdt.hoursInDay);

// GetStartOfDay for |tomorrow| fails.
zdt = new Temporal.ZonedDateTime(864n * 10n**19n, "-01");
assert.throws(RangeError, () => zdt.hoursInDay);
