// Copyright (C) 2018 Bloomberg LP. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.with
description: Test behaviour around DST boundaries with the option offset set.
features: [Temporal]
includes: [temporalHelpers.js]
---*/

const dstStartDay = Temporal.PlainDateTime.from("2000-04-02T12:00:01").toZonedDateTime("America/Vancouver");
const dstEndDay = Temporal.PlainDateTime.from("2000-10-29T12:00:01").toZonedDateTime("America/Vancouver");
const oneThirty = {
  hour: 1,
  minute: 30
};
const twoThirty = {
  hour: 2,
  minute: 30
};

// Offset options
const bogus = {
  ...twoThirty,
  offset: "+23:59"
};

const preserveExactSpring = dstStartDay.with(bogus, { offset: "use" });

TemporalHelpers.assertZonedDateTimesEqual(
  preserveExactSpring,
  Temporal.ZonedDateTime.from("2000-03-31T18:31:01-08:00[America/Vancouver]"),
  "Option offset: use, with bogus offset, changes to the exact");
assert.sameValue(
  preserveExactSpring.epochNanoseconds,
  Temporal.Instant.from("2000-04-02T02:30:01+23:59").epochNanoseconds);

assert.throws(RangeError, () => dstStartDay.with({
  ...twoThirty,
  offset: "+23:59"
}, { offset: "reject" }),
  "Option offset: reject, with bogus offset, throws");

const doubleTime = new Temporal.ZonedDateTime(972811801_000_000_000n, "America/Vancouver");
// Option offset: use changes to the exact time with the offset
const preserveExactFall = doubleTime.with({ offset: "-07:00" }, { offset: "use" });
assert.sameValue(preserveExactFall.offset, "-07:00");
assert.sameValue(preserveExactFall.epochNanoseconds, Temporal.Instant.from("2000-10-29T01:30:01-07:00").epochNanoseconds);

// Option offset: prefer adjusts offset of repeated clock time
assert.sameValue(doubleTime.with({ offset: "-07:00" }, { offset: "prefer" }).offset, "-07:00");

// Option offset: reject adjusts offset of repeated clock time
assert.sameValue(doubleTime.with({ offset: "-07:00" }, { offset: "reject" }).offset, "-07:00");

// Option offset: use does not cause the offset to change when adjusting repeated clock time
assert.sameValue(doubleTime.with({ minute: 31 }, { offset: "use" }).offset, "-08:00");

// Option offset: ignore may cause the offset to change when adjusting repeated clock time
assert.sameValue(doubleTime.with({ minute: 31 }, { offset: "ignore" }).offset, "-07:00");

// Option offset: prefer does not cause the offset to change when adjusting repeated clock time
assert.sameValue(doubleTime.with({ minute: 31 }, { offset: "prefer" }).offset, "-08:00");

// Option offset: reject does not cause the offset to change when adjusting repeated clock time
assert.sameValue(doubleTime.with({ minute: 31 }, { offset: "reject" }).offset, "-08:00");
