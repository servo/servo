// Copyright (C) 2024 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.prototype.tozoneddatetime
description: >
  GetEpochNanosecondsFor throws a RangeError for values outside the valid limits.
info: |
  Temporal.PlainDate.prototype.toZonedDateTime ( item )

  ...
  5. If temporalTime is undefined, then
    ...
  6. Else,
    ...
    d. Let epochNs be ? GetEpochNanosecondsFor(timeZone, isoDateTime, compatible).
  ...
features: [Temporal]
---*/

var minDate = new Temporal.PlainDate(-271821, 4, 19);
var minDateTime = new Temporal.PlainDate(-271821, 4, 20);
var maxDate = new Temporal.PlainDate(275760, 9, 13);

var midnight = new Temporal.PlainTime();
var oneHourPastMidnight = new Temporal.PlainTime(1);

assert.throws(RangeError, () => minDate.toZonedDateTime({
  timeZone: "UTC",
  plainTime: oneHourPastMidnight,
}));

assert.throws(RangeError, () => minDate.toZonedDateTime({
  timeZone: "+00",
  plainTime: oneHourPastMidnight,
}));

assert.throws(RangeError, () => minDateTime.toZonedDateTime({
  timeZone: "+01",
  temporalTime: midnight,
}));

assert.throws(RangeError, () => maxDate.toZonedDateTime({
  timeZone: "-01",
  temporalTime: midnight,
}));
