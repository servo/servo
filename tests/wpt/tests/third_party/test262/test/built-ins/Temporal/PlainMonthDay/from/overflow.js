// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainmonthday.from
description: Handling for overflow option
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const validValues = [
  new Temporal.PlainMonthDay(5, 2),
  "05-02",
];
validValues.forEach((value) => {
  const constrain = Temporal.PlainMonthDay.from(value, { overflow: "constrain" });
  TemporalHelpers.assertPlainMonthDay(constrain, "M05", 2, "overflow is ignored: constrain");

  const reject = Temporal.PlainMonthDay.from(value, { overflow: "reject" });
  TemporalHelpers.assertPlainMonthDay(reject, "M05", 2, "overflow is ignored: reject");
});

const propertyBag1 = { year: 2000, month: 13, day: 34 };
const result1 = Temporal.PlainMonthDay.from(propertyBag1, { overflow: "constrain" });
TemporalHelpers.assertPlainMonthDay(result1, "M12", 31, "default overflow is constrain");
assert.throws(RangeError, () => Temporal.PlainMonthDay.from(propertyBag1, { overflow: "reject" }),
  "invalid property bag: reject");

const propertyBag2 = { month: 1, day: 32 };
const result2 = Temporal.PlainMonthDay.from(propertyBag2, { overflow: "constrain" });
TemporalHelpers.assertPlainMonthDay(result2, "M01", 31, "default overflow is constrain");
assert.throws(RangeError, () => Temporal.PlainMonthDay.from(propertyBag2, { overflow: "reject" }),
  "invalid property bag: reject");

assert.throws(RangeError, () => Temporal.PlainMonthDay.from("13-34", { overflow: "constrain" }),
  "invalid ISO string: constrain");
assert.throws(RangeError, () => Temporal.PlainMonthDay.from("13-34", { overflow: "reject" }),
  "invalid ISO string: reject");

const opt = { overflow: "constrain" };

let result = Temporal.PlainMonthDay.from({ year: 2021, month: 13, day: 1 }, opt);
TemporalHelpers.assertPlainMonthDay(result, "M12", 1, "month 13 is constrained to 12");

result = Temporal.PlainMonthDay.from({ year: 2021, month: 999999, day: 500 }, opt);
TemporalHelpers.assertPlainMonthDay(result, "M12", 31, "month 999999 is constrained to 12 and day 500 is constrained to 31");

[-99999, -1, 0].forEach((month) => {
  assert.throws(
    RangeError,
    () => Temporal.PlainMonthDay.from({ year: 2021, month, day: 1 }, opt),
    `Month ${month} is out of range for 2021 even with overflow: constrain`
  );
});

TemporalHelpers.ISOMonths.forEach(({ month, monthCode, daysInMonth }) => {
  const day = daysInMonth + 1;

  result = Temporal.PlainMonthDay.from({ month, day }, opt);
  TemporalHelpers.assertPlainMonthDay(result, monthCode, daysInMonth,
    `day is constrained from ${day} to ${daysInMonth} in month ${month}`);

  result = Temporal.PlainMonthDay.from({ month, day: 9001 }, opt);
  TemporalHelpers.assertPlainMonthDay(result, monthCode, daysInMonth,
    `day is constrained to ${daysInMonth} in month ${month}`);

  result = Temporal.PlainMonthDay.from({ monthCode, day }, opt);
  TemporalHelpers.assertPlainMonthDay(result, monthCode, daysInMonth,
    `day is constrained from ${day} to ${daysInMonth} in monthCode ${monthCode}`);

  result = Temporal.PlainMonthDay.from({ monthCode, day: 9001 }, opt);
  TemporalHelpers.assertPlainMonthDay(result, monthCode, daysInMonth,
    `day is constrained to ${daysInMonth} in monthCode ${monthCode}`);
});

[ ["month", 2], ["monthCode", "M02"] ].forEach(([ name, value ]) => {
  result = Temporal.PlainMonthDay.from({ year: 2020, [name]: value, day: 30 }, opt);
  TemporalHelpers.assertPlainMonthDay(result, "M02", 29, `${name} ${value} is constrained to 29 in leap year 2020`);

  result = Temporal.PlainMonthDay.from({ year: 2021, [name]: value, day: 29 }, opt);
  TemporalHelpers.assertPlainMonthDay(result, "M02", 28, `${name} ${value} is constrained to 28 in common year 2021`);
});

[-1, 0, 13, 9995].forEach((month) => {
  assert.throws(
    RangeError,
    () => Temporal.PlainMonthDay.from({year: 2021, month, day: 5}, { overflow: "reject" }),
    `Month ${month} is out of range for 2021 with overflow: reject`
  );
});

[-1, 0, 32, 999].forEach((day) => {
  assert.throws(
    RangeError,
    () => Temporal.PlainMonthDay.from({ year: 2021, month: 12, day }, { overflow: "reject" }),
    `Day ${day} is out of range for 2021-12 with overflow: reject`
  );
  assert.throws(
    RangeError,
    () => Temporal.PlainMonthDay.from({ monthCode: "M12", day }, { overflow: "reject" }),
    `Day ${day} is out of range for 2021-M12 with overflow: reject`
  );
});

TemporalHelpers.ISOMonths.forEach(({ month, monthCode, daysInMonth }) => {
  const day = daysInMonth + 1;
  assert.throws(RangeError,
    () => Temporal.PlainMonthDay.from({ month, day }, { overflow: "reject" }),
    `Day ${day} is out of range for month ${month} with overflow: reject`);
  assert.throws(RangeError,
    () => Temporal.PlainMonthDay.from({ monthCode, day }, { overflow: "reject" }),
    `Day ${day} is out of range for monthCode ${monthCode} with overflow: reject`);
});

[ ["month", 2], ["monthCode", "M02"] ].forEach(([ name, value ]) => {
  assert.throws(RangeError,
    () => Temporal.PlainMonthDay.from({ year: 2020, [name]: value, day: 30 }, { overflow: "reject" }),
    `Day 30 is out of range for ${name} ${value} in leap year 2020 with overflow: reject`);
  assert.throws(RangeError,
    () => Temporal.PlainMonthDay.from({ year: 2021, [name]: value, day: 29 }, { overflow: "reject" }),
    `Day 29 is out of range for ${name} ${value} in common year 2021 with overflow: reject`);
});
