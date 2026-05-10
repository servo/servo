// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.instant.prototype.tostring
description: Basic tests for toString().
features: [BigInt, Temporal]
---*/

const afterEpoch = new Temporal.Instant(217175010_123_456_789n);
assert.sameValue(afterEpoch.toString(), "1976-11-18T14:23:30.123456789Z", "basic toString() after epoch");

const beforeEpoch = new Temporal.Instant(-217175010_876_543_211n);
assert.sameValue(beforeEpoch.toString(), "1963-02-13T09:36:29.123456789Z", "basic toString() before epoch");
