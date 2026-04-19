// Copyright (C) 2021 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainmonthday.from
description: Returns correctly with valid data.
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const options = [
  { overflow: "constrain" },
  { overflow: "reject" },
  {},
  undefined,
];
options.forEach((opt) => {
  const optionsDesc = opt && JSON.stringify(opt);
  let result = Temporal.PlainMonthDay.from({ year: 2021, month: 7, day: 3 }, opt);
  TemporalHelpers.assertPlainMonthDay(result, "M07", 3, `month 7, day 3, with year, options = ${optionsDesc}`);
  result = Temporal.PlainMonthDay.from({ year: 2021, month: 12, day: 31 }, opt);
  TemporalHelpers.assertPlainMonthDay(result, "M12", 31, `month 12, day 31, with year, options = ${optionsDesc}`);
  result = Temporal.PlainMonthDay.from({ monthCode: "M07", day: 3 }, opt);
  TemporalHelpers.assertPlainMonthDay(result, "M07", 3, `monthCode M07, day 3, options = ${optionsDesc}`);
  result = Temporal.PlainMonthDay.from({ monthCode: "M12", day: 31 }, opt);
  TemporalHelpers.assertPlainMonthDay(result, "M12", 31, `monthCode M12, day 31, options = ${optionsDesc}`);
});

TemporalHelpers.ISOMonths.forEach(({ month, monthCode, daysInMonth }) => {
  let result = Temporal.PlainMonthDay.from({ month, day: daysInMonth });
  TemporalHelpers.assertPlainMonthDay(result, monthCode, daysInMonth, `month ${month}, day ${daysInMonth}`);

  result = Temporal.PlainMonthDay.from({ monthCode, day: daysInMonth });
  TemporalHelpers.assertPlainMonthDay(result, monthCode, daysInMonth, `monthCode ${monthCode}, day ${daysInMonth}`);
});
