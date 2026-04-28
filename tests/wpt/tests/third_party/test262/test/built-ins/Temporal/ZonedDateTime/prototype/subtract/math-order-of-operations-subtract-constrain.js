// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.add
description: Math order of operations - add / reject.
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const breakoutUnits = (op, zdt, d, options) => zdt[op]({ years: d.years }, options)[op]({ months: d.months }, options)[op]({ weeks: d.weeks }, options)[op]({ days: d.days }, options)[op]({
  hours: d.hours,
  minutes: d.minutes,
  seconds: d.seconds,
  milliseconds: d.milliseconds,
  microseconds: d.microseconds,
  nanoseconds: d.nanoseconds
}, options);

// const zdt = Temporal.ZonedDateTime.from("2020-03-31T00:00-08:00[-08:00]");
const zdt = new Temporal.ZonedDateTime(1585641600000000000n, "-08:00");
const d = new Temporal.Duration(0, 1, 0, 1, 0, 0, 0, 0, 0, 0);
const options = { overflow: "constrain" };
// "2020-02-28T00:00:00-08:00[-08:00]"
const expected = new Temporal.ZonedDateTime(1582876800000000000n, "-08:00");

const result = zdt.subtract(d, options);
TemporalHelpers.assertZonedDateTimesEqual(result, expected);
TemporalHelpers.assertZonedDateTimesEqual(breakoutUnits("subtract", zdt, d, options),
                                          result);

