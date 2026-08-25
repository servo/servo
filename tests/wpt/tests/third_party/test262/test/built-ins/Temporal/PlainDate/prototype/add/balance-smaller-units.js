// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.calendar.prototype.dateadd
description: Durations with units smaller than days are balanced before adding, in the calendar
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const date = new Temporal.PlainDate(2000, 5, 2);
const duration = new Temporal.Duration(0, 0, 0, 1, 24, 1440, 86400, 86400_000, 86400_000_000, 86400_000_000_000);

const result = date.add(duration);
TemporalHelpers.assertPlainDate(result, 2000, 5, "M05", 9, "units smaller than days are balanced");

const resultString = date.add("P1DT24H1440M86400S");
TemporalHelpers.assertPlainDate(resultString, 2000, 5, "M05", 6, "units smaller than days are balanced");

const resultPropBag = date.add({ days: 1, hours: 24, minutes: 1440, seconds: 86400, milliseconds: 86400_000, microseconds: 86400_000_000, nanoseconds: 86400_000_000_000 });
TemporalHelpers.assertPlainDate(resultPropBag, 2000, 5, "M05", 9, "units smaller than days are balanced");

const negativeDuration = new Temporal.Duration(0, 0, 0, -1, -24, -1440, -86400, -86400_000, -86400_000_000, -86400_000_000_000);
const resultNegative = date.add(negativeDuration);
TemporalHelpers.assertPlainDate(resultNegative, 2000, 4, "M04", 25, "units smaller than days are balanced");
