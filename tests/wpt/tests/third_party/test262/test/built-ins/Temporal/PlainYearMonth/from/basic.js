// Copyright (C) 2021 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.from
description: Returns correctly with valid data
includes: [temporalHelpers.js]
features: [Temporal]
---*/

let result = Temporal.PlainYearMonth.from({ year: 2021, month: 7 });
TemporalHelpers.assertPlainYearMonth(result, 2021, 7, "M07", "year 2021, month 7");
result = Temporal.PlainYearMonth.from({ year: 2021, month: 12 });
TemporalHelpers.assertPlainYearMonth(result, 2021, 12, "M12", "year 2021, month 12");
result = Temporal.PlainYearMonth.from({ year: 2021, monthCode: "M07" });
TemporalHelpers.assertPlainYearMonth(result, 2021, 7, "M07", "year 2021, monthCode M07");
result = Temporal.PlainYearMonth.from({ year: 2021, monthCode: "M12" });
TemporalHelpers.assertPlainYearMonth(result, 2021, 12, "M12", "year 2021, monthCode M12");

["constrain", "reject"].forEach((overflow) => {
  const opt = { overflow };
  result = Temporal.PlainYearMonth.from({ year: 2021, month: 7 }, opt);
  TemporalHelpers.assertPlainYearMonth(result, 2021, 7, "M07", `year 2021, month 7, overflow ${overflow}`);
  result = Temporal.PlainYearMonth.from({ year: 2021, month: 12 }, opt);
  TemporalHelpers.assertPlainYearMonth(result, 2021, 12, "M12", `year 2021, month 12, overflow ${overflow}`);
  result = Temporal.PlainYearMonth.from({ year: 2021, monthCode: "M07" }, opt);
  TemporalHelpers.assertPlainYearMonth(result, 2021, 7, "M07", `year 2021, monthCode M07, overflow ${overflow}`);
  result = Temporal.PlainYearMonth.from({ year: 2021, monthCode: "M12" }, opt);
  TemporalHelpers.assertPlainYearMonth(result, 2021, 12, "M12", `year 2021, monthCode M12, overflow ${overflow}`);
});
