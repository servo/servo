// Copyright (C) 2020 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.from
description: Property bag is converted to Duration; Duration is copied
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const d1 = Temporal.Duration.from({ milliseconds: 1000, month: 1 });
TemporalHelpers.assertDuration(d1, 0, 0, 0, 0, 0, 0, 0, 1000, 0, 0);

const d2 = Temporal.Duration.from(d1);
assert.notSameValue(d1, d2);
TemporalHelpers.assertDuration(d1, 0, 0, 0, 0, 0, 0, 0, 1000, 0, 0);
TemporalHelpers.assertDuration(d2, 0, 0, 0, 0, 0, 0, 0, 1000, 0, 0);
