// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.prototype.since
description: Fallback value for smallestUnit option
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const earlier = new Temporal.PlainDate(2000, 5, 2);
const later = new Temporal.PlainDate(2001, 6, 3);

const explicit = later.since(earlier, { smallestUnit: undefined });
TemporalHelpers.assertDuration(explicit, 0, 0, 0, 397, 0, 0, 0, 0, 0, 0, "default smallestUnit is day");
const implicit = later.since(earlier, {});
TemporalHelpers.assertDuration(implicit, 0, 0, 0, 397, 0, 0, 0, 0, 0, 0, "default smallestUnit is day");
const lambda = later.since(earlier, () => {});
TemporalHelpers.assertDuration(lambda, 0, 0, 0, 397, 0, 0, 0, 0, 0, 0, "default smallestUnit is day");
