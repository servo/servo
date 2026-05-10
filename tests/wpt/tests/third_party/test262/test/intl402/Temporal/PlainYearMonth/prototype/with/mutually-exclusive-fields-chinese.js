// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.prototype.with
description: Calendar-specific mutually exclusive keys in the chinese calendar
info: NonIsoFieldKeysToIgnore ( _calendar_, _keys_ )
includes: [temporalHelpers.js]
features: [Temporal, Intl.Era-monthcode]
---*/

const options = { overflow: "reject" };
const instance = Temporal.PlainYearMonth.from({ year: 1981, monthCode: "M12", calendar: "chinese" }, options);

TemporalHelpers.assertPlainYearMonth(
  instance,
  1981, 12, "M12",
  "check that all fields are as expected",
  undefined, undefined, null
);

TemporalHelpers.assertPlainYearMonth(
  instance.with({ month: 5 }, options),
  1981, 5, "M05",
  "month excludes monthCode",
  undefined, undefined, null
);

TemporalHelpers.assertPlainYearMonth(
  instance.with({ monthCode: "M05" }, options),
  1981, 5, "M05",
  "monthCode excludes month",
  undefined, undefined, null
);

assert.throws(
  TypeError,
  () => instance.with({ eraYear: 2025, era: "ce" }),
  "eraYear and era are invalid for this calendar",
);

