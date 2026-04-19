// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.from
description: |
  Creating a Duration from an ISO 8601 string with an absent designator between
  two other designators
includes: [temporalHelpers.js]
features: [Temporal]
---*/

// Date designators: years-weeks (months missing)

const d1 = Temporal.Duration.from("P3Y4W");
TemporalHelpers.assertDuration(d1, 3, 0, 4, 0, 0, 0, 0, 0, 0, 0, "years-weeks string");

// Date designators: years-days (months and weeks missing)

const d2 = Temporal.Duration.from("P3Y4D");
TemporalHelpers.assertDuration(d2, 3, 0, 0, 4, 0, 0, 0, 0, 0, 0, "years-days string");

// Date designators: months-days (weeks missing)

const d3 = Temporal.Duration.from("P3M4D");
TemporalHelpers.assertDuration(d3, 0, 3, 0, 4, 0, 0, 0, 0, 0, 0, "months-days string");

// Time designators: hours-seconds (minutes missing)

const d4 = Temporal.Duration.from("PT3H4.123456789S");
TemporalHelpers.assertDuration(d4, 0, 0, 0, 0, 3, 0, 4, 123, 456, 789, "hours-seconds string");
