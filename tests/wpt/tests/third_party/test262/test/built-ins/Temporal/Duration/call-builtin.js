// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration
description: Constructor should not call built-in functions.
includes: [temporalHelpers.js]
features: [Temporal]
---*/

Number.isFinite = () => { throw new Test262Error("should not call Number.isFinite") };
Math.sign = () => { throw new Test262Error("should not call Math.sign") };

const duration = new Temporal.Duration(1, 1);
TemporalHelpers.assertDuration(duration, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0);
