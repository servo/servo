// Copyright (C) 2018 Bloomberg LP. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.round
description: >
  Temporal.ZonedDateTime.prototype.round rounds correctly with respect to dst
   boundaries.
features: [Temporal]
---*/

// Rounds correctly to a 25-hour day.
// The 12.5 hour is the halfway point, which is 11:30 local time, since
// 2:00-2:59 repeats.
var roundTo = { smallestUnit: "day" };
var roundMeDown = Temporal.PlainDateTime.from("2000-10-29T11:29:59").toZonedDateTime("America/Vancouver");
assert.sameValue(`${ roundMeDown.round(roundTo) }`, "2000-10-29T00:00:00-07:00[America/Vancouver]");
var roundMeUp = Temporal.PlainDateTime.from("2000-10-29T11:30:01").toZonedDateTime("America/Vancouver");
assert.sameValue(`${ roundMeUp.round(roundTo) }`, "2000-10-30T00:00:00-08:00[America/Vancouver]");

// Rounds correctly to a 23-hour day.
// The 11.5 hour is the halfway point, which is 12:30 local time,
// since 2:00-2:59 skips.
var roundTo = { smallestUnit: "day" };
var roundMeDown = Temporal.PlainDateTime.from("2000-04-02T12:29:59").toZonedDateTime("America/Vancouver");
assert.sameValue(`${ roundMeDown.round(roundTo) }`, "2000-04-02T00:00:00-08:00[America/Vancouver]");
var roundMeUp = Temporal.PlainDateTime.from("2000-04-02T12:30:01").toZonedDateTime("America/Vancouver");
assert.sameValue(`${ roundMeUp.round(roundTo) }`, "2000-04-03T00:00:00-07:00[America/Vancouver]");

// Rounding up to a nonexistent wall-clock time.
var almostSkipped = Temporal.PlainDateTime.from("2000-04-02T01:59:59.999999999").toZonedDateTime("America/Vancouver");
var rounded = almostSkipped.round({
  smallestUnit: "microsecond",
  roundingMode: "halfExpand"
});
assert.sameValue(`${ rounded }`, "2000-04-02T03:00:00-07:00[America/Vancouver]");
assert.sameValue(rounded.epochNanoseconds - almostSkipped.epochNanoseconds, 1n);
