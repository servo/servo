// Copyright (C) 2021 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-temporal.plaindate.from
description: Property bag with year, monthCode and day
info: |
  1. Let calendar be the this value.
  2. Perform ? RequireInternalSlot(calendar, [[InitializedTemporalCalendar]]).
  3. Assert: calendar.[[Identifier]] is "iso8601".
  4. If Type(fields) is not Object, throw a TypeError exception.
  5. Set options to ? GetOptionsObject(options).
  6. Let result be ? ISODateFromFields(fields, options).
  7. Return ? CreateTemporalDate(result.[[Year]], result.[[Month]], result.[[Day]], calendar).
features: [Temporal]
includes: [temporalHelpers.js]
---*/

TemporalHelpers.assertPlainDate(
    Temporal.PlainDate.from({year: 2021, monthCode: "M07", day: 15}),
    2021, 7, "M07", 15,
    "year/monthCode/day");
