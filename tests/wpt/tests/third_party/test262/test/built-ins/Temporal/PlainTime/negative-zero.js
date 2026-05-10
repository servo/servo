// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaintime
description: Negative zero arguments are treated as zero.
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const plainTime = new Temporal.PlainTime(-0, -0, -0, -0, -0, -0);
TemporalHelpers.assertPlainTime(plainTime, 0, 0, 0, 0, 0, 0);
