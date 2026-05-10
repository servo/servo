// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.instant.prototype.until
description: Options may be a function object.
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const feb20 = Temporal.Instant.from("2020-02-01T00:00Z");
const feb21 = Temporal.Instant.from("2021-02-01T00:00Z");

TemporalHelpers.assertDuration(feb20.until(feb21, () => {}),
                               0, 0, 0, 0, 0, 0, 31622400, 0, 0, 0);
