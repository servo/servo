// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.with
description: Calendar-specific mutually exclusive keys in the persian calendar
info: NonIsoFieldKeysToIgnore ( _calendar_, _keys_ )
includes: [temporalHelpers.js]
features: [Temporal, Intl.Era-monthcode]
---*/

const options = { overflow: "reject" };
const instance = Temporal.PlainDateTime.from({ year: 1404, monthCode: "M12", day: 15, hour: 12, minute: 34, calendar: "persian" }, options);

TemporalHelpers.assertPlainDateTime(
  instance,
  1404, 12, "M12", 15, 12, 34, 0, 0, 0, 0,
  "check that all fields are as expected",
  /* era = */ "ap", /* eraYear = */ 1404
);

TemporalHelpers.assertPlainDateTime(
  instance.with({ era: "ap", eraYear: 1400 }, options),
  1400, 12, "M12", 15, 12, 34, 0, 0, 0, 0,
  "era and eraYear together exclude year",
  "ap", 1400
);

TemporalHelpers.assertPlainDateTime(
  instance.with({ year: 1402 }, options),
  1402, 12, "M12", 15, 12, 34, 0, 0, 0, 0,
  "year excludes era and eraYear",
  "ap", 1402
);

TemporalHelpers.assertPlainDateTime(
  instance.with({ month: 5 }, options),
  1404, 5, "M05", 15, 12, 34, 0, 0, 0, 0,
  "month excludes monthCode",
  "ap", 1404
);

TemporalHelpers.assertPlainDateTime(
  instance.with({ monthCode: "M05" }, options),
  1404, 5, "M05", 15, 12, 34, 0, 0, 0, 0,
  "monthCode excludes month",
  "ap", 1404
);

assert.throws(
  TypeError,
  () => instance.with({ eraYear: 1403 }),
  "eraYear excludes year and era, and cannot be provided without era",
);

assert.throws(
  TypeError,
  () => instance.with({ era: "ap" }),
  "era excludes year and eraYear, and cannot be provided without eraYear",
);
