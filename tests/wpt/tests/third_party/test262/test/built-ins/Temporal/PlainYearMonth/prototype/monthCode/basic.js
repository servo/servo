// Copyright (C) 2026 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.prototype.monthcode
description: monthCode property for PlainYearMonth
features: [Temporal]
---*/

assert.sameValue((new Temporal.PlainYearMonth(1999, 6)).monthCode, 'M06');
