// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.prototype.with
description: Throw if the argument has a calendar field
features: [Temporal]
---*/

const ym = Temporal.PlainYearMonth.from("2019-10");
assert.throws(TypeError, () => ym.with({ year: 2021, calendar: "iso8601" }));
