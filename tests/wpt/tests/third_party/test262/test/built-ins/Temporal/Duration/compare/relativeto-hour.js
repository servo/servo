// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.compare
description: relativeTo with hours.
features: [Temporal]
---*/

const oneDay = new Temporal.Duration(0, 0, 0, 1);
const hours24 = new Temporal.Duration(0, 0, 0, 0, 24);
assert.sameValue(Temporal.Duration.compare(oneDay, hours24),
  0,
  "relativeTo not required for days");
assert.sameValue(
  Temporal.Duration.compare(oneDay, hours24, { relativeTo: Temporal.PlainDate.from("2017-01-01") }),
  0,
  "relativeTo does not affect days if PlainDate");
assert.sameValue(Temporal.Duration.compare(oneDay, hours24, { relativeTo: "2019-11-03" }),
  0,
  "casts relativeTo to PlainDate from string");
assert.sameValue(Temporal.Duration.compare(oneDay, hours24, { relativeTo: { year: 2019, month: 11, day: 3 } }),
  0,
  "casts relativeTo to PlainDate from object");
