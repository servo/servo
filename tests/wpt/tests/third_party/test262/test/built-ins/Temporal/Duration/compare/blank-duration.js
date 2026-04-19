// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.compare
description: Behaviour with blank durations
features: [Temporal]
---*/

const blank1 = new Temporal.Duration();
const blank2 = new Temporal.Duration();
const { compare } = Temporal.Duration;
const plainRelativeTo = new Temporal.PlainDate(2025, 8, 22);
const zonedRelativeTo = new Temporal.ZonedDateTime(1n, "UTC");

assert.sameValue(compare(blank1, blank2), 0, "zero durations compared without relativeTo");
assert.sameValue(compare(blank1, blank2, { relativeTo: plainRelativeTo }), 0,
  "zero durations compared with PlainDate relativeTo");
assert.sameValue(compare(blank1, blank2, { relativeTo: zonedRelativeTo }), 0,
  "zero durations compared with ZonedDateTime relativeTo");
