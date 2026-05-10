// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.with
description: Calendar-specific mutually exclusive keys in the dangi calendar
info: NonIsoFieldKeysToIgnore ( _calendar_, _keys_ )
includes: [temporalHelpers.js]
features: [Temporal, Intl.Era-monthcode]
---*/

const options = { overflow: "reject" };
const instance = Temporal.ZonedDateTime.from({ year: 1981, monthCode: "M12", day: 15, hour: 12, minute: 34, timeZone: "UTC", calendar: "dangi" }, options);

TemporalHelpers.assertPlainDateTime(
  instance.toPlainDateTime(),
  1981, 12, "M12", 15, 12, 34, 0, 0, 0, 0,
  "check that all fields are as expected"
);

TemporalHelpers.assertPlainDateTime(
  instance.with({ month: 5 }, options).toPlainDateTime(),
  1981, 5, "M05", 15, 12, 34, 0, 0, 0, 0,
  "month excludes monthCode"
);

TemporalHelpers.assertPlainDateTime(
  instance.with({ monthCode: "M05" }, options).toPlainDateTime(),
  1981, 5, "M05", 15, 12, 34, 0, 0, 0, 0,
  "monthCode excludes month"
);

assert.throws(
  TypeError,
  () => instance.with({ eraYear: 2025, era: "ce" }),
  "eraYear and era are invalid for this calendar",
);

