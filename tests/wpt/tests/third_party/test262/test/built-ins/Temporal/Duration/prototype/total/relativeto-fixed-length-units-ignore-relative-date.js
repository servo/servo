// Copyright (C) 2018 Bloomberg LP. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.prototype.total
description: relativeTo option not required to round fixed-length units in durations without variable units.
features: [Temporal]
---*/

const d2 = new Temporal.Duration(0, 0, 0, 5, 5, 5, 5, 5, 5, 5);
const d2Nanoseconds = d2.days * 24 * 3600000000000 + d2.hours * 3600000000000 + d2.minutes * 60000000000 + d2.seconds * 1000000000 + d2.milliseconds * 1000000 + d2.microseconds * 1000 + d2.nanoseconds;
const totalD2 = {
  days: d2Nanoseconds / (24 * 3600000000000),
  hours: d2Nanoseconds / 3600000000000,
  minutes: d2Nanoseconds / 60000000000,
  seconds: d2Nanoseconds / 1000000000,
  milliseconds: d2Nanoseconds / 1000000,
  microseconds: d2Nanoseconds / 1000,
  nanoseconds: d2Nanoseconds
};

assert(Math.abs(d2.total({ unit: "days" }) - totalD2.days) < Number.EPSILON);
assert(Math.abs(d2.total({ unit: "hours" }) - totalD2.hours) < Number.EPSILON);
assert(Math.abs(d2.total({ unit: "minutes" }) - totalD2.minutes) < Number.EPSILON);
assert(Math.abs(d2.total({ unit: "seconds" }) - totalD2.seconds) < Number.EPSILON);
assert(Math.abs(d2.total({ unit: "milliseconds" }) - totalD2.milliseconds) < Number.EPSILON);
assert(Math.abs(d2.total({ unit: "microseconds" }) - totalD2.microseconds) < Number.EPSILON);
assert.sameValue(d2.total({ unit: "nanoseconds" }), totalD2.nanoseconds);

const negativeD2 = d2.negated();
assert(Math.abs(negativeD2.total({ unit: "days" }) - -totalD2.days) < Number.EPSILON);
assert(Math.abs(negativeD2.total({ unit: "hours" }) - -totalD2.hours) < Number.EPSILON);
assert(Math.abs(negativeD2.total({ unit: "minutes" }) - -totalD2.minutes) < Number.EPSILON);
assert(Math.abs(negativeD2.total({ unit: "seconds" }) - -totalD2.seconds) < Number.EPSILON);
assert(Math.abs(negativeD2.total({ unit: "milliseconds" }) - -totalD2.milliseconds) < Number.EPSILON);
assert(Math.abs(negativeD2.total({ unit: "microseconds" }) - -totalD2.microseconds) < Number.EPSILON);
assert.sameValue(negativeD2.total({ unit: "nanoseconds" }), -totalD2.nanoseconds);
