// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.with
description: Calendar-specific mutually exclusive keys in the buddhist calendar
info: NonIsoFieldKeysToIgnore ( _calendar_, _keys_ )
includes: [temporalHelpers.js]
features: [Temporal, Intl.Era-monthcode]
---*/

const options = { overflow: "reject" };
const instance = Temporal.PlainDateTime.from({ year: 2566, monthCode: "M12", day: 15, hour: 12, minute: 34, calendar: "buddhist" }, options);

TemporalHelpers.assertPlainDateTime(
  instance,
  2566, 12, "M12", 15, 12, 34, 0, 0, 0, 0,
  "check that all fields are as expected",
  /* era = */ "be", /* eraYear = */ 2566
);

TemporalHelpers.assertPlainDateTime(
  instance.with({ era: "be", eraYear: 2560 }, options),
  2560, 12, "M12", 15, 12, 34, 0, 0, 0, 0,
  "era and eraYear together exclude year",
  "be", 2560
);

TemporalHelpers.assertPlainDateTime(
  instance.with({ year: 2559 }, options),
  2559, 12, "M12", 15, 12, 34, 0, 0, 0, 0,
  "year excludes era and eraYear",
  "be", 2559
);

TemporalHelpers.assertPlainDateTime(
  instance.with({ month: 5 }, options),
  2566, 5, "M05", 15, 12, 34, 0, 0, 0, 0,
  "month excludes monthCode",
  "be", 2566
);

TemporalHelpers.assertPlainDateTime(
  instance.with({ monthCode: "M05" }, options),
  2566, 5, "M05", 15, 12, 34, 0, 0, 0, 0,
  "monthCode excludes month",
  "be", 2566
);

assert.throws(
  TypeError,
  () => instance.with({ eraYear: 2560 }),
  "eraYear excludes year and era, and cannot be provided without era",
);

assert.throws(
  TypeError,
  () => instance.with({ era: "be" }),
  "era excludes year and eraYear, and cannot be provided without eraYear",
);
