// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.prototype.from
description: islamic calendar name is not supported
features: [Temporal, Intl.Era-monthcode]
---*/

const calendar = "islamic";

assert.throws(RangeError, () =>
  Temporal.PlainYearMonth.from({year: 1500, month: 1, day: 1, calendar}),
  "fallback for calendar ID 'islamic' only supported in Intl.DateTimeFormat constructor, not Temporal"
);
