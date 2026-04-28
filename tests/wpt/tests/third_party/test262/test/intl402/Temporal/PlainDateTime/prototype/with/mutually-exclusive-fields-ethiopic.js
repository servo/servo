// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.with
description: Calendar-specific mutually exclusive keys in the ethiopic calendar
info: NonIsoFieldKeysToIgnore ( _calendar_, _keys_ )
includes: [temporalHelpers.js]
features: [Temporal, Intl.Era-monthcode]
---*/

const options = { overflow: "reject" };
const instance = Temporal.PlainDateTime.from({ year: 2018, monthCode: "M12", day: 15, hour: 12, minute: 34, calendar: "ethiopic" }, options);

TemporalHelpers.assertPlainDateTime(
  instance,
  2018, 12, "M12", 15, 12, 34, 0, 0, 0, 0,
  "check that all fields are as expected",
  /* era = */ "am", /* eraYear = */ 2018
);

TemporalHelpers.assertPlainDateTime(
  instance.with({ era: "aa", eraYear: 5500 }, options),
  0, 12, "M12", 15, 12, 34, 0, 0, 0, 0,
  "era and eraYear together exclude year",
  "aa", 5500
);

TemporalHelpers.assertPlainDateTime(
  instance.with({ year: -2 }, options),
  -2, 12, "M12", 15, 12, 34, 0, 0, 0, 0,
  "year excludes era and eraYear",
  "aa", 5498
);

TemporalHelpers.assertPlainDateTime(
  instance.with({ month: 5 }, options),
  2018, 5, "M05", 15, 12, 34, 0, 0, 0, 0,
  "month excludes monthCode",
  "am", 2018
);

TemporalHelpers.assertPlainDateTime(
  instance.with({ monthCode: "M05" }, options),
  2018, 5, "M05", 15, 12, 34, 0, 0, 0, 0,
  "monthCode excludes month",
  "am", 2018
);

assert.throws(
  TypeError,
  () => instance.with({ eraYear: 1 }),
  "eraYear excludes year and era, and cannot be provided without era",
);

assert.throws(
  TypeError,
  () => instance.with({ era: "aa" }),
  "era excludes year and eraYear, and cannot be provided without eraYear",
);
