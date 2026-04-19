// Copyright (C) 2021 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainmonthday.from
description: Throw RangeError for an out-of-range, conflicting, or ill-formed monthCode
features: [Temporal]
---*/

["m1", "M1", "m01"].forEach((monthCode) => {
  assert.throws(RangeError, () => Temporal.PlainMonthDay.from({ monthCode, day: 17 }),
    `monthCode '${monthCode}' is not well-formed (without numeric month)`);
  assert.throws(RangeError, () => Temporal.PlainMonthDay.from({ month: 1, monthCode, day: 17 }),
    `monthCode '${monthCode}' is not well-formed (with numeric month)`);
});

assert.throws(RangeError, () => Temporal.PlainMonthDay.from({ year: 2021, month: 12, monthCode: "M11", day: 17 }),
  "monthCode and month conflict");

["M00", "M19", "M99", "M13", "M00L", "M05L", "M13L"].forEach((monthCode) => {
  assert.throws(RangeError, () => Temporal.PlainMonthDay.from({ monthCode, day: 17 }),
    `monthCode '${monthCode}' is not valid for ISO 8601 calendar (without numeric month)`);
  var monthNumber = Number(monthCode.slice(1, 3)) + (monthCode.length - 3);
  assert.throws(
    RangeError,
    () => Temporal.PlainMonthDay.from({ month: monthNumber, monthCode, day: 17 }),
    `monthCode '${monthCode}' is not valid for ISO 8601 calendar (with numeric month)`
  );
  var clampedMonthNumber = monthNumber < 1 ? 1 : monthNumber > 12 ? 12 : monthNumber;
  assert.throws(
    RangeError,
    () => Temporal.PlainMonthDay.from({ month: clampedMonthNumber, monthCode, day: 17 }),
    `monthCode '${monthCode}' is not valid for ISO 8601 calendar (with clamped numeric month)`
  );
});

assert.throws(
  RangeError,
  () => Temporal.PlainMonthDay.from({ day: 1, monthCode: "L99M", year: Symbol() }),
  "Month code syntax is validated before year type is validated"
);

assert.throws(
  TypeError,
  () => Temporal.PlainMonthDay.from({ day: 1, monthCode: "M99L", year: Symbol() }),
  "Month code suitability is validated after year type is validated"
);
