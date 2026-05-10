// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.with
description: Calendar-specific mutually exclusive keys in the indian calendar
info: NonIsoFieldKeysToIgnore ( _calendar_, _keys_ )
includes: [temporalHelpers.js]
features: [Temporal, Intl.Era-monthcode]
---*/

const options = { overflow: "reject" };
const instance = Temporal.ZonedDateTime.from({ year: 1947, monthCode: "M12", day: 15, hour: 12, minute: 34, timeZone: "UTC", calendar: "indian" }, options);

TemporalHelpers.assertPlainDateTime(
  instance.toPlainDateTime(),
  1947, 12, "M12", 15, 12, 34, 0, 0, 0, 0,
  "check that all fields are as expected",
  /* era = */ "shaka", /* eraYear = */ 1947
);

TemporalHelpers.assertPlainDateTime(
  instance.with({ era: "shaka", eraYear: 1940 }, options).toPlainDateTime(),
  1940, 12, "M12", 15, 12, 34, 0, 0, 0, 0,
  "era and eraYear together exclude year",
  "shaka", 1940
);

TemporalHelpers.assertPlainDateTime(
  instance.with({ year: 1943 }, options).toPlainDateTime(),
  1943, 12, "M12", 15, 12, 34, 0, 0, 0, 0,
  "year excludes era and eraYear",
  "shaka", 1943
);

TemporalHelpers.assertPlainDateTime(
  instance.with({ month: 5 }, options).toPlainDateTime(),
  1947, 5, "M05", 15, 12, 34, 0, 0, 0, 0,
  "month excludes monthCode",
  "shaka", 1947
);

TemporalHelpers.assertPlainDateTime(
  instance.with({ monthCode: "M05" }, options).toPlainDateTime(),
  1947, 5, "M05", 15, 12, 34, 0, 0, 0, 0,
  "monthCode excludes month",
  "shaka", 1947
);

assert.throws(
  TypeError,
  () => instance.with({ eraYear: 1940 }),
  "eraYear excludes year and era, and cannot be provided without era",
);

assert.throws(
  TypeError,
  () => instance.with({ era: "shaka" }),
  "era excludes year and eraYear, and cannot be provided without eraYear",
);
