// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.prototype.with
description: Calendar-specific mutually exclusive keys in the islamic-civil calendar
info: NonIsoFieldKeysToIgnore ( _calendar_, _keys_ )
includes: [temporalHelpers.js]
features: [Temporal, Intl.Era-monthcode]
---*/

const options = { overflow: "reject" };
const instance = Temporal.PlainDate.from({ year: 1447, monthCode: "M12", day: 15, calendar: "islamic-civil" }, options);

TemporalHelpers.assertPlainDate(
  instance,
  1447, 12, "M12", 15,
  "check that all fields are as expected",
  /* era = */ "ah", /* eraYear = */ 1447
);

TemporalHelpers.assertPlainDate(
  instance.with({ era: "bh", eraYear: 1 }, options),
  0, 12, "M12", 15,
  "era and eraYear together exclude year",
  "bh", 1
);

TemporalHelpers.assertPlainDate(
  instance.with({ year: -2 }, options),
  -2, 12, "M12", 15,
  "year excludes era and eraYear",
  "bh", 3
);

TemporalHelpers.assertPlainDate(
  instance.with({ month: 5 }, options),
  1447, 5, "M05", 15,
  "month excludes monthCode",
  "ah", 1447
);

TemporalHelpers.assertPlainDate(
  instance.with({ monthCode: "M05" }, options),
  1447, 5, "M05", 15,
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
