// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.with
description: Calendar-specific mutually exclusive keys in the islamic-tbla calendar
info: NonIsoFieldKeysToIgnore ( _calendar_, _keys_ )
includes: [temporalHelpers.js]
features: [Temporal, Intl.Era-monthcode]
---*/

const options = { overflow: "reject" };
const instance = Temporal.PlainDateTime.from({ year: 1447, monthCode: "M12", day: 15, hour: 12, minute: 34, calendar: "islamic-tbla" }, options);

TemporalHelpers.assertPlainDateTime(
  instance,
  1447, 12, "M12", 15, 12, 34, 0, 0, 0, 0,
  "check that all fields are as expected",
  /* era = */ "ah", /* eraYear = */ 1447
);

TemporalHelpers.assertPlainDateTime(
  instance.with({ era: "bh", eraYear: 1 }, options),
  0, 12, "M12", 15, 12, 34, 0, 0, 0, 0,
  "era and eraYear together exclude year",
  "bh", 1
);

TemporalHelpers.assertPlainDateTime(
  instance.with({ year: -2 }, options),
  -2, 12, "M12", 15, 12, 34, 0, 0, 0, 0,
  "year excludes era and eraYear",
  "bh", 3
);

TemporalHelpers.assertPlainDateTime(
  instance.with({ month: 5 }, options),
  1447, 5, "M05", 15, 12, 34, 0, 0, 0, 0,
  "month excludes monthCode",
  "ah", 1447
);

TemporalHelpers.assertPlainDateTime(
  instance.with({ monthCode: "M05" }, options),
  1447, 5, "M05", 15, 12, 34, 0, 0, 0, 0,
  "monthCode excludes month",
  "ah", 1447
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
