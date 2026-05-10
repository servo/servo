// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaintime
description: Basic tests for the PlainTime constructor.
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const args = [15, 23, 30, 123, 456, 789];
const plainTime = new Temporal.PlainTime(...args);
TemporalHelpers.assertPlainTime(plainTime, ...args);
