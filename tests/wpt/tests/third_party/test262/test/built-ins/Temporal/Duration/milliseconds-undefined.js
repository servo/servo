// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration
description: Undefined arguments should be treated as zero.
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const args = [1, 1, 1, 1, 1, 1, 1];

const explicit = new Temporal.Duration(...args, undefined);
TemporalHelpers.assertDuration(explicit, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, "explicit");

const implicit = new Temporal.Duration(...args);
TemporalHelpers.assertDuration(implicit, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, "implicit");
