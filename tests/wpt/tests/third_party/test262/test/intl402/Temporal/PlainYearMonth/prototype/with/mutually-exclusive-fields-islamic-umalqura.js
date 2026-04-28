// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.prototype.with
description: Calendar-specific mutually exclusive keys in the islamic-umalqura calendar
info: NonIsoFieldKeysToIgnore ( _calendar_, _keys_ )
includes: [temporalHelpers.js]
features: [Temporal, Intl.Era-monthcode]
---*/

const options = { overflow: "reject" };
const instance = Temporal.PlainYearMonth.from({ year: 1447, monthCode: "M12", calendar: "islamic-umalqura" }, options);

TemporalHelpers.assertPlainYearMonth(
  instance,
  1447, 12, "M12",
  "check that all fields are as expected",
  /* era = */ "ah", /* eraYear = */ 1447, null
);

TemporalHelpers.assertPlainYearMonth(
  instance.with({ era: "bh", eraYear: 1 }, options),
  0, 12, "M12",
  "era and eraYear together exclude year",
  "bh", 1, null
);

TemporalHelpers.assertPlainYearMonth(
  instance.with({ year: -2 }, options),
  -2, 12, "M12",
  "year excludes era and eraYear",
  "bh", 3, null
);

TemporalHelpers.assertPlainYearMonth(
  instance.with({ month: 5 }, options),
  1447, 5, "M05",
  "month excludes monthCode",
  "ah", 1447, null
);

TemporalHelpers.assertPlainYearMonth(
  instance.with({ monthCode: "M05" }, options),
  1447, 5, "M05",
  "monthCode excludes month",
  "ah", 1447, null
);

assert.throws(
  TypeError,
  () => instance.with({ eraYear: 1 }),
  "eraYear excludes year and era, and cannot be provided without era",
);

assert.throws(
  TypeError,
  () => instance.with({ era: "bh" }),
  "era excludes year and eraYear, and cannot be provided without eraYear",
);
