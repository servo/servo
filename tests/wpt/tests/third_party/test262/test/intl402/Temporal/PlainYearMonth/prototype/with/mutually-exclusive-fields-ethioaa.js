// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.prototype.with
description: Calendar-specific mutually exclusive keys in the ethioaa calendar
info: NonIsoFieldKeysToIgnore ( _calendar_, _keys_ )
includes: [temporalHelpers.js]
features: [Temporal, Intl.Era-monthcode]
---*/

const options = { overflow: "reject" };
const instance = Temporal.PlainYearMonth.from({ year: 7518, monthCode: "M12", calendar: "ethioaa" }, options);

TemporalHelpers.assertPlainYearMonth(
  instance,
  7518, 12, "M12",
  "check that all fields are as expected",
  /* era = */ "aa", /* eraYear = */ 7518, null
);

TemporalHelpers.assertPlainYearMonth(
  instance.with({ era: "aa", eraYear: 7515 }, options),
  7515, 12, "M12",
  "era and eraYear together exclude year",
  "aa", 7515, null
);

TemporalHelpers.assertPlainYearMonth(
  instance.with({ year: 7510 }, options),
  7510, 12, "M12",
  "year excludes era and eraYear",
  "aa", 7510, null
);

TemporalHelpers.assertPlainYearMonth(
  instance.with({ month: 5 }, options),
  7518, 5, "M05",
  "month excludes monthCode",
  "aa", 7518, null
);

TemporalHelpers.assertPlainYearMonth(
  instance.with({ monthCode: "M05" }, options),
  7518, 5, "M05",
  "monthCode excludes month",
  "aa", 7518, null
);

assert.throws(
  TypeError,
  () => instance.with({ eraYear: 7512 }),
  "eraYear excludes year and era, and cannot be provided without era",
);

assert.throws(
  TypeError,
  () => instance.with({ era: "aa" }),
  "era excludes year and eraYear, and cannot be provided without eraYear",
);
