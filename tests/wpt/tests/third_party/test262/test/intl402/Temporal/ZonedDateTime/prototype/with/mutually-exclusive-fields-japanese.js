// Copyright (C) 2023 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.with
description: Calendar-specific mutually exclusive keys in the japanese calendar
info: NonIsoFieldKeysToIgnore ( _calendar_, _keys_ )
includes: [temporalHelpers.js]
features: [Temporal, Intl.Era-monthcode]
---*/

const options = { overflow: "reject" };
const lastDayOfShowa = Temporal.ZonedDateTime.from({ era: "showa", eraYear: 64, year: 1989, month: 1, monthCode: "M01", day: 7, hour: 12, minute: 34, timeZone: "UTC", calendar: "japanese" }, options);

TemporalHelpers.assertPlainDateTime(
  lastDayOfShowa.toPlainDateTime(),
  1989, 1, "M01", 7, 12, 34, 0, 0, 0, 0,
  "check expected fields",
  /* era = */ "showa", /* eraYear = */ 64
);

TemporalHelpers.assertPlainDateTime(
  lastDayOfShowa.with({ day: 10 }, options).toPlainDateTime(),
  1989, 1, "M01", 10, 12, 34, 0, 0, 0, 0,
  "day excludes era and eraYear",
  /* era = */ "heisei", /* eraYear = */ 1
);

TemporalHelpers.assertPlainDateTime(
  lastDayOfShowa.with({ month: 2 }, options).toPlainDateTime(),
  1989, 2, "M02", 7, 12, 34, 0, 0, 0, 0,
  "month excludes monthCode, era, and eraYear",
  "heisei", 1
);

TemporalHelpers.assertPlainDateTime(
  lastDayOfShowa.with({ monthCode: "M03" }, options).toPlainDateTime(),
  1989, 3, "M03", 7, 12, 34, 0, 0, 0, 0,
  "monthCode excludes month, era, and eraYear",
  "heisei", 1
);

TemporalHelpers.assertPlainDateTime(
  lastDayOfShowa.with({ year: 1988 }, options).toPlainDateTime(),
  1988, 1, "M01", 7, 12, 34, 0, 0, 0, 0,
  "year excludes era and eraYear (within same era)",
  "showa", 63
);

TemporalHelpers.assertPlainDateTime(
  lastDayOfShowa.with({ year: 1990 }, options).toPlainDateTime(),
  1990, 1, "M01", 7, 12, 34, 0, 0, 0, 0,
  "year excludes era and eraYear (in a different era)",
  "heisei", 2
);

assert.throws(
  TypeError,
  () => lastDayOfShowa.with({ eraYear: 1 }),
  "eraYear excludes year and era, and cannot be provided without era",
);

assert.throws(
  TypeError,
  () => lastDayOfShowa.with({ era: "heisei" }),
  "era excludes year and eraYear, and cannot be provided without eraYear",
);
