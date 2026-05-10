// Copyright (C) 2018 Bloomberg LP. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
includes: [temporalHelpers.js]
esid: sec-temporal.duration
description: Minimum value is zero.
features: [Temporal]
---*/

TemporalHelpers.assertDuration(new Temporal.Duration(0, 0, 0, 0, 0, 0, 0, 0, 0, 0),
                               0, 0, 0, 0, 0, 0, 0, 0, 0, 0);
