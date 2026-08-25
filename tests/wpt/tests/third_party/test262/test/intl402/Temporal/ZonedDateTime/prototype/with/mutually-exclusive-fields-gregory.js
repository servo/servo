// Copyright (C) 2023 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.with
description: Calendar-specific mutually exclusive keys in the gregory calendar
info: NonIsoFieldKeysToIgnore ( _calendar_, _keys_ )
includes: [temporalHelpers.js]
features: [Temporal, Intl.Era-monthcode]
---*/

const options = { overflow: "reject" };
const instance = Temporal.ZonedDateTime.from({ year: 1981, monthCode: "M12", day: 15, hour: 12, minute: 34, timeZone: "UTC", calendar: "gregory" }, options);

TemporalHelpers.assertPlainDateTime(
  instance.toPlainDateTime(),
  1981, 12, "M12", 15, 12, 34, 0, 0, 0, 0,
  "check that all fields are as expected",
  /* era = */ "ce", /* eraYear = */ 1981
);

TemporalHelpers.assertPlainDateTime(
  instance.with({ era: "bce", eraYear: 1 }, options).toPlainDateTime(),
  0, 12, "M12", 15, 12, 34, 0, 0, 0, 0,
  "era and eraYear together exclude year",
  "bce", 1
);

TemporalHelpers.assertPlainDateTime(
  instance.with({ year: -2 }, options).toPlainDateTime(),
  -2, 12, "M12", 15, 12, 34, 0, 0, 0, 0,
  "year excludes era and eraYear",
  "bce", 3
);

TemporalHelpers.assertPlainDateTime(
  instance.with({ month: 5 }, options).toPlainDateTime(),
  1981, 5, "M05", 15, 12, 34, 0, 0, 0, 0,
  "month excludes monthCode",
  "ce", 1981
);

TemporalHelpers.assertPlainDateTime(
  instance.with({ monthCode: "M05" }, options).toPlainDateTime(),
  1981, 5, "M05", 15, 12, 34, 0, 0, 0, 0,
  "monthCode excludes month",
  "ce", 1981
);

assert.throws(
  TypeError,
  () => instance.with({ eraYear: 1 }),
  "eraYear excludes year and era, and cannot be provided without era",
);

assert.throws(
  TypeError,
  () => instance.with({ era: "bce" }),
  "era excludes year and eraYear, and cannot be provided without eraYear",
);
