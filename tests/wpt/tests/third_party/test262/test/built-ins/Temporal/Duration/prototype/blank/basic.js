// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-get-temporal.duration.prototype.blank
description: Basic tests for blank.
features: [Temporal]
---*/

assert.sameValue(Temporal.Duration.from("P3DT1H").blank, false);
assert.sameValue(Temporal.Duration.from("-PT2H20M30S").blank, false);
assert.sameValue(Temporal.Duration.from("PT0S").blank, true);
const zero = Temporal.Duration.from({
  years: 0,
  months: 0,
  weeks: 0,
  days: 0,
  hours: 0,
  minutes: 0,
  seconds: 0,
  milliseconds: 0,
  microseconds: 0,
  nanoseconds: 0
});
assert.sameValue(zero.blank, true);
assert(new Temporal.Duration().blank, "created via constructor");
