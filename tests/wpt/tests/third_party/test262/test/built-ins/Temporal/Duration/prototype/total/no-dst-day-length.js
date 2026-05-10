// Copyright (C) 2024 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-temporal.duration.prototype.total
description: Total day length is 24 hours when not affected by DST
features: [Temporal]
---*/

const oneDay = new Temporal.Duration(0, 0, 0, 1);
const hours48 = new Temporal.Duration(0, 0, 0, 0, 48);

assert.sameValue(oneDay.total("hours"), 24, "with no relativeTo, days are 24 hours");
assert.sameValue(hours48.total({ unit: "days" }), 2, "with no relativeTo, 48 hours = 2 days");

const plainRelativeTo = new Temporal.PlainDate(2017, 1, 1);

assert.sameValue(oneDay.total({ unit: "hours", relativeTo: plainRelativeTo }), 24,
  "with PlainDate relativeTo, days are 24 hours");
assert.sameValue(hours48.total({ unit: "days", relativeTo: plainRelativeTo }), 2,
  "with PlainDate relativeTo, 48 hours = 2 days")

const zonedRelativeTo = new Temporal.ZonedDateTime(1_000_000_000_000_000_000n, "+04:30");

assert.sameValue(oneDay.total({ unit: "hours", relativeTo: zonedRelativeTo }), 24,
  "with ZonedDateTime relativeTo, days are 24 hours if the duration encompasses no DST change");
assert.sameValue(hours48.total({ unit: "days", relativeTo: zonedRelativeTo }), 2,
  "with ZonedDateTime relativeTo, 48 hours = 2 days if the duration encompasses no DST change");
