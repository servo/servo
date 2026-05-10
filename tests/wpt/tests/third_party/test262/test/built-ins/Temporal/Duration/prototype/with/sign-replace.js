// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.prototype.with
description: Replacing the sign is supported.
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const d = Temporal.Duration.from({ years: 5, days: 1 });
assert.sameValue(d.sign, 1, "original sign");
const d2 = d.with({ years: -1, days: 0, minutes: -1 });
assert.sameValue(d2.sign, -1, "new sign");
TemporalHelpers.assertDuration(d2, -1, 0, 0, 0, 0, -1, 0, 0, 0, 0);
