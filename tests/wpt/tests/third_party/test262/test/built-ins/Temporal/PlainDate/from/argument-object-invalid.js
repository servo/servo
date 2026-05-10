// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.from
description: Property bag is correctly converted into PlainDate
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const badFields = { year: 2019, month: 1, day: 32 };
assert.throws(RangeError, () => Temporal.PlainDate.from(badFields, { overflow: "reject" }),
  "bad fields with reject");
TemporalHelpers.assertPlainDate(Temporal.PlainDate.from(badFields),
  2019, 1, "M01", 31, "bad fields with missing overflow");
TemporalHelpers.assertPlainDate(Temporal.PlainDate.from(badFields, { overflow: "constrain" }),
  2019, 1, "M01", 31, "bad fields with constrain");

assert.throws(RangeError,
  () => Temporal.PlainDate.from({ year: 1976, month: 11, monthCode: "M12", day: 18 }),
  "month and monthCode must agree");

assert.throws(TypeError,
  () => Temporal.PlainDate.from({ year: 2019, day: 15 }),
  "missing month");

assert.throws(TypeError,
  () => Temporal.PlainDate.from({}),
  "no properties");

assert.throws(TypeError,
  () => Temporal.PlainDate.from({ month: 12 }),
  "missing year, day");

assert.throws(TypeError,
  () => Temporal.PlainDate.from({ year: 1976, months: 11, day: 18 }),
  "misspelled month");

assert.throws(TypeError,
  () => Temporal.PlainDate.from({ year: undefined, month: 11, day: 18 }),
  "year undefined");
