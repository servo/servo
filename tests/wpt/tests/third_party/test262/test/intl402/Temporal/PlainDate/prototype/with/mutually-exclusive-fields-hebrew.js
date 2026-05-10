// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.prototype.with
description: Calendar-specific mutually exclusive keys in the hebrew calendar
info: NonIsoFieldKeysToIgnore ( _calendar_, _keys_ )
includes: [temporalHelpers.js]
features: [Temporal, Intl.Era-monthcode]
---*/

const options = { overflow: "reject" };
const instance = Temporal.PlainDate.from({ year: 5786, monthCode: "M12", day: 15, calendar: "hebrew" }, options);

TemporalHelpers.assertPlainDate(
  instance,
  5786, 12, "M12", 15,
  "check that all fields are as expected",
  /* era = */ "am", /* eraYear = */ 5786
);

TemporalHelpers.assertPlainDate(
  instance.with({ era: "am", eraYear: 5760 }, options),
  5760, 13, "M12", 15,
  "era and eraYear together exclude year",
  "am", 5760
);

TemporalHelpers.assertPlainDate(
  instance.with({ year: 5784 }, options),
  5784, 13, "M12", 15,
  "year excludes era and eraYear",
  "am", 5784
);

TemporalHelpers.assertPlainDate(
  instance.with({ month: 5 }, options),
  5786, 5, "M05", 15,
  "month excludes monthCode",
  "am", 5786
);

TemporalHelpers.assertPlainDate(
  instance.with({ monthCode: "M05" }, options),
  5786, 5, "M05", 15,
  "monthCode excludes month",
  "am", 5786
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
