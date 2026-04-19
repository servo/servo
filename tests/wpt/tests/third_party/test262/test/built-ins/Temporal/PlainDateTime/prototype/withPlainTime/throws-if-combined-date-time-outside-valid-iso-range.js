// Copyright (C) 2024 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.withplaintime
description: >
  Throws if combined date-time outside valid ISO date range.
info: |
  Temporal.PlainDateTime.prototype.withPlainTime ( [ plainTimeLike ] )

  1. Let dateTime be the this value.
  ...
  2. Let time be ? ToTimeRecordOrMidnight(plainTimeLike).
  4. Let isoDateTime be CombineISODateAndTimeRecord(dateTime.[[ISODateTime]].[[ISODate]], time).
  5. Return ? CreateTemporalDateTime(isoDateTime, dateTime.[[Calendar]]).
features: [Temporal]
---*/

var minDateTime = new Temporal.PlainDateTime(-271821, 4, 19, 0, 0, 0, 0, 0, 1);
var midnight = new Temporal.PlainTime();

assert.throws(RangeError, () => minDateTime.withPlainTime(midnight));
