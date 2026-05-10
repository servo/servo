// Copyright (C) 2023 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.add
description: Throws when ambiguous result with reject
features: [Temporal]
---*/

// "2020-01-31T15:00-08:00[-08:00]"
const jan31 = new Temporal.ZonedDateTime(1580511600000000000n, "-08:00");

assert.throws(RangeError, () => jan31.add({ months: 1 }, { overflow: "reject" }));
