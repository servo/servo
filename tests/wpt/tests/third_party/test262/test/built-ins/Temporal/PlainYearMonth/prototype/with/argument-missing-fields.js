// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.prototype.with
description: TypeError thrown when argument doesn't contain any of the supported properties
features: [Temporal]
---*/

const ym = Temporal.PlainYearMonth.from("2019-10");
assert.throws(TypeError, () => ym.with({}), "No properties");
assert.throws(TypeError, () => ym.with({ months: 12 }), "Only plural 'months' property");
