// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.with
description: Calendar-specific mutually exclusive keys in the ethioaa calendar
info: NonIsoFieldKeysToIgnore ( _calendar_, _keys_ )
includes: [temporalHelpers.js]
features: [Temporal, Intl.Era-monthcode]
---*/

const options = { overflow: "reject" };
const instance = Temporal.ZonedDateTime.from({ year: 7518, monthCode: "M12", day: 15, hour: 12, minute: 34, timeZone: "UTC", calendar: "ethioaa" }, options);

TemporalHelpers.assertPlainDateTime(
  instance.toPlainDateTime(),
  7518, 12, "M12", 15, 12, 34, 0, 0, 0, 0,
  "check that all fields are as expected",
  /* era = */ "aa", /* eraYear = */ 7518
);

TemporalHelpers.assertPlainDateTime(
  instance.with({ era: "aa", eraYear: 7515 }, options).toPlainDateTime(),
  7515, 12, "M12", 15, 12, 34, 0, 0, 0, 0,
  "era and eraYear together exclude year",
  "aa", 7515
);

TemporalHelpers.assertPlainDateTime(
  instance.with({ year: 7510 }, options).toPlainDateTime(),
  7510, 12, "M12", 15, 12, 34, 0, 0, 0, 0,
  "year excludes era and eraYear",
  "aa", 7510
);

TemporalHelpers.assertPlainDateTime(
  instance.with({ month: 5 }, options).toPlainDateTime(),
  7518, 5, "M05", 15, 12, 34, 0, 0, 0, 0,
  "month excludes monthCode",
  "aa", 7518
);

TemporalHelpers.assertPlainDateTime(
  instance.with({ monthCode: "M05" }, options).toPlainDateTime(),
  7518, 5, "M05", 15, 12, 34, 0, 0, 0, 0,
  "monthCode excludes month",
  "aa", 7518
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
