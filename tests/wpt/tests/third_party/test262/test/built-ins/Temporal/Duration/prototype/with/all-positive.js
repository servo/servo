// Copyright (C) 2021 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.prototype.with
description: >
  Returns a correctly merged object when the argument replaces the fields with
  all positive values.
info: |
  1. Let duration be the this value.
  2. Perform ? RequireInternalSlot(duration, [[InitializedTemporalDuration]]).
  3. Let temporalDurationLike be ? ToPartialDuration(temporalDurationLike).
  4. If temporalDurationLike.[[Years]] is not undefined, then
    a. Let years be temporalDurationLike.[[Years]].
  5. Else,
    a. Let years be duration.[[Years]].
  6. If temporalDurationLike.[[Months]] is not undefined, then
    a. Let months be temporalDurationLike.[[Months]].
  7. Else,
    a. Let months be duration.[[Months]].
  8. If temporalDurationLike.[[Weeks]] is not undefined, then
    a. Let weeks be temporalDurationLike.[[Weeks]].
  9. Else,
    a. Let weeks be duration.[[Weeks]].
  10. If temporalDurationLike.[[Days]] is not undefined, then
    a. Let days be temporalDurationLike.[[Days]].
  11. Else,
    a. Let days be duration.[[Days]].
  12. If temporalDurationLike.[[Hours]] is not undefined, then
    a. Let hours be temporalDurationLike.[[Hours]].
  13. Else,
    a. Let hours be duration.[[Hours]].
  14. If temporalDurationLike.[[Minutes]] is not undefined, then
    a. Let minutes be temporalDurationLike.[[Minutes]].
  15. Else,
    a. Let minutes be duration.[[Minutes]].
  16. If temporalDurationLike.[[Seconds]] is not undefined, then
    a. Let seconds be temporalDurationLike.[[Seconds]].
  17. Else,
    a. Let seconds be duration.[[Seconds]].
  18. If temporalDurationLike.[[Milliseconds]] is not undefined, then
    a. Let milliseconds be temporalDurationLike.[[Milliseconds]].
  19. Else,
    a. Let milliseconds be duration.[[Milliseconds]].
  20. If temporalDurationLike.[[Microseconds]] is not undefined, then
    a. Let microseconds be temporalDurationLike.[[Microseconds]].
  21. Else,
    a. Let microseconds be duration.[[Microseconds]].
  22. If temporalDurationLike.[[Nanoseconds]] is not undefined, then
    a. Let nanoseconds be temporalDurationLike.[[Nanoseconds]].
  23. Else,
    a. Let nanoseconds be duration.[[Nanoseconds]].
  24. Return ? CreateTemporalDuration(years, months, weeks, days, hours, minutes, seconds, milliseconds, microseconds, nanoseconds).
features: [Temporal]
includes: [temporalHelpers.js]
---*/

const argAllPositive = {
  years: 9,
  months: 8,
  weeks: 7,
  days: 6,
  hours: 5,
  minutes: 4,
  seconds: 3,
  milliseconds: 2,
  microseconds: 1,
  nanoseconds: 10,
};

const d1 = new Temporal.Duration();
TemporalHelpers.assertDuration(
  d1.with(argAllPositive), 9, 8, 7, 6, 5, 4, 3, 2, 1, 10,
  "replace all zeroes with all positive"
);

const d2 = new Temporal.Duration(1, 2, 3, 4, 5, 6, 7, 8, 9, 10);
TemporalHelpers.assertDuration(
  d2.with(argAllPositive), 9, 8, 7, 6, 5, 4, 3, 2, 1, 10,
  "replace all positive with all positive");

const d3 = new Temporal.Duration(1e5, 2e5, 3e5, 4e5, 5e5, 6e5, 7e5, 8e5, 9e5, 10e5);
TemporalHelpers.assertDuration(
  d3.with(argAllPositive), 9, 8, 7, 6, 5, 4, 3, 2, 1, 10,
  "replace all positive large numbers with all positive"
);

const d4 = new Temporal.Duration(-1, -2, -3, -4, -5, -6, -7, -8, -9, -10);
TemporalHelpers.assertDuration(
  d4.with(argAllPositive), 9, 8, 7, 6, 5, 4, 3, 2, 1, 10,
  "replace all negative with all positive"
);
