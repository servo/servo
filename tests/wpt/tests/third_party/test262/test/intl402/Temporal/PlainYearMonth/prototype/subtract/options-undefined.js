// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.prototype.subtract
description: Verify that undefined options are handled correctly.
features: [Temporal]
---*/

// overflow option has no effect on addition in the ISO calendar, so verify this
// with a lunisolar calendar. Default overflow is "constrain" so this should not
// throw.

const yearmonth = Temporal.PlainYearMonth.from({
  year: 5779,
  monthCode: "M05L",
  calendar: "hebrew"
});
const duration = { years: 1 };

yearmonth.subtract(duration, undefined);

yearmonth.subtract(duration);
