// Copyright (C) 2026 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-get-temporal.zoneddatetime.prototype.monthsinyear
description: Checking months in year for a "normal" case (non-undefined, non-boundary case, etc.)
features: [Temporal]
---*/

const zdt = new Temporal.ZonedDateTime(217178610123456789n, "UTC");
assert.sameValue(zdt.monthsInYear, 12, "check months in year information");
assert.sameValue((new Temporal.ZonedDateTime(-23205637787000000000n, "UTC")).monthsInYear, 12);
