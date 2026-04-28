// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.compare
description: Throws if a value in the relativeTo property bag is missing.
features: [Temporal]
---*/

const oneDay = new Temporal.Duration(0, 0, 0, 1);
const hours24 = new Temporal.Duration(0, 0, 0, 0, 24);
assert.throws(TypeError, () => Temporal.Duration.compare(oneDay, hours24, { relativeTo: { month: 11, day: 3 } }), "missing year");
assert.throws(TypeError, () => Temporal.Duration.compare(oneDay, hours24, { relativeTo: { year: 2019, month: 11 } }), "missing day");
assert.throws(TypeError, () => Temporal.Duration.compare(oneDay, hours24, { relativeTo: { year: 2019, day: 3 } }), "missing month");
