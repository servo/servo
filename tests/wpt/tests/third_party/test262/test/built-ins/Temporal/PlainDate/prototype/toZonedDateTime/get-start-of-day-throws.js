// Copyright (C) 2024 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.prototype.tozoneddatetime
description: >
  GetStartOfDay throws a RangeError for values outside the valid limits.
info: |
  Temporal.PlainDate.prototype.toZonedDateTime ( item )

  ...
  5. If temporalTime is undefined, then
    a. Let epochNs be ? GetStartOfDay(timeZone, temporalDate.[[ISODate]]).
  ...
features: [Temporal]
---*/

var minDate = new Temporal.PlainDate(-271821, 4, 19);
var minDateTime = new Temporal.PlainDate(-271821, 4, 20);
var maxDate = new Temporal.PlainDate(275760, 9, 13);

assert.throws(RangeError, () => minDate.toZonedDateTime("UTC"));
assert.throws(RangeError, () => minDate.toZonedDateTime("+00"));
assert.throws(RangeError, () => minDateTime.toZonedDateTime("+01"));
assert.throws(RangeError, () => maxDate.toZonedDateTime("-01"));
