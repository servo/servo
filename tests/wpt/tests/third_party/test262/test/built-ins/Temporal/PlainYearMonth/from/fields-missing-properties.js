// Copyright (C) 2021 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.from
description: Throws TypeError with incorrect input data type
features: [Temporal]
---*/

assert.throws(TypeError, () => Temporal.PlainYearMonth.from({}), "at least one correctly spelled property is required");
assert.throws(TypeError, () => Temporal.PlainYearMonth.from({ month: 1 }), "year is required");
assert.throws(TypeError, () => Temporal.PlainYearMonth.from({ year: 2021 }), "month or monthCode is required");
