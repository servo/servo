// Copyright (C) 2024 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.prototype.tozoneddatetime
description: >
  Throws if combined date-time outside valid ISO date range.
info: |
  Temporal.PlainDate.prototype.toZonedDateTime ( item )

  1. Let temporalDate be the this value.
  ...
  5. If temporalTime is undefined, then
    ...
  6. Else,
    a. Set temporalTime to ? ToTemporalTime(temporalTime).
    b. Let isoDateTime be CombineISODateAndTimeRecord(temporalDate.[[ISODate]], temporalTime.[[Time]]).
    c. If ISODateTimeWithinLimits(isoDateTime) is false, throw a RangeError exception.
    ...
features: [Temporal]
---*/

var minDate = new Temporal.PlainDate(-271821, 4, 19);
var midnight = new Temporal.PlainTime();
var item = {
  timeZone: "+00",
  plainTime: midnight,
};

assert.throws(RangeError, () => minDate.toZonedDateTime(item));
