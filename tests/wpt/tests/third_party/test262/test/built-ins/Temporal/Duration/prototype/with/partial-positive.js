// Copyright (C) 2021 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.prototype.with
description: >
  Returns a correctly merged object when the argument replaces only some of the
  fields with positive values.
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

const durationlike1 = { years: 9, hours: 5 };
const durationlike2 = { months: 8, minutes: 4 };
const durationlike3 = { weeks: 7, seconds: 3 };
const durationlike4 = { days: 6, milliseconds: 2 };
const durationlike5 = { microseconds: 987, nanoseconds: 123 };

const d1 = new Temporal.Duration();
TemporalHelpers.assertDuration(
    d1.with(durationlike1), 9, 0, 0, 0, 5, 0, 0, 0, 0, 0, "replace all zeroes with years and hours");
TemporalHelpers.assertDuration(
    d1.with(durationlike2), 0, 8, 0, 0, 0, 4, 0, 0, 0, 0, "replace all zeroes wtih months and minutes");
TemporalHelpers.assertDuration(
    d1.with(durationlike3), 0, 0, 7, 0, 0, 0, 3, 0, 0, 0, "replace all zeroes with weeks and seconds");
TemporalHelpers.assertDuration(
    d1.with(durationlike4), 0, 0, 0, 6, 0, 0, 0, 2, 0, 0, "replace all zeroes with days and milliseconds");
TemporalHelpers.assertDuration(
    d1.with(durationlike5), 0, 0, 0, 0, 0, 0, 0, 0, 987, 123, "replace all zeroes with microseconds and nanoseconds");

const d2 = new Temporal.Duration(1, 2, 3, 4, 5, 6, 7, 8, 9, 10);
TemporalHelpers.assertDuration(
    d2.with(durationlike1), 9, 2, 3, 4, 5, 6, 7, 8, 9, 10, "replace all positive with years and hours");
TemporalHelpers.assertDuration(
    d2.with(durationlike2), 1, 8, 3, 4, 5, 4, 7, 8, 9, 10, "replace all positive with months and minutes");
TemporalHelpers.assertDuration(
    d2.with(durationlike3), 1, 2, 7, 4, 5, 6, 3, 8, 9, 10, "replace all positive with weeks and seconds");
TemporalHelpers.assertDuration(
    d2.with(durationlike4), 1, 2, 3, 6, 5, 6, 7, 2, 9, 10, "replace all positive with days and milliseconds");
TemporalHelpers.assertDuration(
    d2.with(durationlike5), 1, 2, 3, 4, 5, 6, 7, 8, 987, 123, "replace all positive with microseconds and nanoseconds");
