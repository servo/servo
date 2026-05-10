// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.prototype.toplaindate
description: Basic check for toPlainDate()
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const ym = Temporal.PlainYearMonth.from("2002-01");
TemporalHelpers.assertPlainDate(ym.toPlainDate({ day: 22 }), 2002, 1, "M01", 22);
assert.throws(TypeError, () => ym.toPlainDate({ something: "nothing" }));
