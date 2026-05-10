// Copyright (C) 2022 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.prototype.total
description: >
  Relative to a ZonedDateTime with a fractional number of days.
features: [Temporal]
---*/

let duration = Temporal.Duration.from({
  weeks: 1,
  days: 0,
  hours: 1,
});

let zdt = new Temporal.ZonedDateTime(0n, "UTC", "iso8601");

let result = duration.total({
  relativeTo: zdt,
  unit: "days",
});

assert.sameValue(result, 7 + 1 / 24);
