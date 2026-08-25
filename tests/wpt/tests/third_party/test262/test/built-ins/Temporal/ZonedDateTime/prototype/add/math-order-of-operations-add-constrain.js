// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.add
description: Math order of operations - add / constrain.
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

// const zdt = Temporal.ZonedDateTime.from("2020-01-31T00:00-08:00[-08:00]");
const zdt = new Temporal.ZonedDateTime(1580457600000000000n, "-08:00");
const d = new Temporal.Duration(0, 1, 0, 1, 0, 0, 0, 0, 0, 0);
// "2020-03-01T00:00:00-08:00[-08:00]"
const expected = new Temporal.ZonedDateTime(1583049600000000000n, "-08:00");

const options = { overflow: "constrain" };
const result = zdt.add(d, options);

TemporalHelpers.assertZonedDateTimesEqual(result, expected);
TemporalHelpers.assertZonedDateTimesEqual(breakoutUnits("add", zdt, d, options), result);

