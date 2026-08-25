// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainmonthday.prototype.from
description: Empty object may be used as options
includes: [temporalHelpers.js]
features: [Temporal]
---*/

TemporalHelpers.assertPlainMonthDay(
  Temporal.PlainMonthDay.from({ monthCode: "M12", day: 15 }, {}), "M12", 15,
  "options may be an empty plain object"
);

TemporalHelpers.assertPlainMonthDay(
  Temporal.PlainMonthDay.from({ monthCode: "M12", day: 15 }, () => {}), "M12", 15,
  "options may be an empty function object"
);
