// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.prototype.with
description: Calendar-specific mutually exclusive keys in the hebrew calendar
info: NonIsoFieldKeysToIgnore ( _calendar_, _keys_ )
includes: [temporalHelpers.js]
features: [Temporal, Intl.Era-monthcode]
---*/

const options = { overflow: "reject" };
const instance = Temporal.PlainYearMonth.from({ year: 5786, monthCode: "M12", calendar: "hebrew" }, options);

TemporalHelpers.assertPlainYearMonth(
  instance,
  5786, 12, "M12",
  "check that all fields are as expected",
  /* era = */ "am", /* eraYear = */ 5786, null
);

TemporalHelpers.assertPlainYearMonth(
  instance.with({ era: "am", eraYear: 5760 }, options),
  5760, 13, "M12",
  "era and eraYear together exclude year",
  "am", 5760, null
);

TemporalHelpers.assertPlainYearMonth(
  instance.with({ year: 5784 }, options),
  5784, 13, "M12",
  "year excludes era and eraYear",
  "am", 5784, null
);

TemporalHelpers.assertPlainYearMonth(
  instance.with({ month: 5 }, options),
  5786, 5, "M05",
  "month excludes monthCode",
  "am", 5786, null
);

TemporalHelpers.assertPlainYearMonth(
  instance.with({ monthCode: "M05" }, options),
  5786, 5, "M05",
  "monthCode excludes month",
  "am", 5786, null
);

assert.throws(
  TypeError,
  () => instance.with({ eraYear: 2560 }),
  "eraYear excludes year and era, and cannot be provided without era",
);

assert.throws(
  TypeError,
  () => instance.with({ era: "am" }),
  "era excludes year and eraYear, and cannot be provided without eraYear",
);
