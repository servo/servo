// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.prototype.from
description: islamic-rgsa calendar name is not supported
features: [Temporal, Intl.Era-monthcode]
---*/

const calendar = "islamic-rgsa";

assert.throws(RangeError, () =>
  Temporal.PlainYearMonth.from({year: 1500, month: 1, calendar}),
  "fallback for calendar ID 'islamic-rgsa' only supported in Intl.DateTimeFormat constructor, not Temporal"
);
