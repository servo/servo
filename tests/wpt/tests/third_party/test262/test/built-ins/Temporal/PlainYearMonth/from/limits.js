// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.from
description: PlainYearMonth.from enforces the supported range
includes: [temporalHelpers.js]
features: [Temporal]
---*/

["reject", "constrain"].forEach((overflow) => {
  [{ year: -271821, month: 3 }, { year: 275760, month: 10 }, "-271821-03", "+275760-10"].forEach((value) => {
    assert.throws(RangeError, () => Temporal.PlainYearMonth.from(value, { overflow }),
      `${JSON.stringify(value)} with ${overflow}`);
  });
});

TemporalHelpers.assertPlainYearMonth(Temporal.PlainYearMonth.from({ year: -271821, month: 4 }),
  -271821, 4, "M04", "min object");
TemporalHelpers.assertPlainYearMonth(Temporal.PlainYearMonth.from({ year: 275760, month: 9 }),
  275760, 9, "M09", "max object");
TemporalHelpers.assertPlainYearMonth(Temporal.PlainYearMonth.from("-271821-04"),
  -271821, 4, "M04", "min string");
TemporalHelpers.assertPlainYearMonth(Temporal.PlainYearMonth.from("+275760-09"),
  275760, 9, "M09", "max string");
