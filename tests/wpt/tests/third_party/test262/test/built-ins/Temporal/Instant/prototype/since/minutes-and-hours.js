// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.instant.prototype.since
description: since() can return minutes and hours.
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const feb20 = Temporal.Instant.from("2020-02-01T00:00Z");
const feb21 = Temporal.Instant.from("2021-02-01T00:00Z");

// can return minutes and hours
TemporalHelpers.assertDuration(feb21.since(feb20, { largestUnit: "hours" }),
                                     0, 0, 0, 0, 8784, 0, 0, 0, 0, 0);
TemporalHelpers.assertDuration(feb21.since(feb20, { largestUnit: "minutes" }),
                                     0, 0, 0, 0, 0, 527040, 0, 0, 0, 0);
