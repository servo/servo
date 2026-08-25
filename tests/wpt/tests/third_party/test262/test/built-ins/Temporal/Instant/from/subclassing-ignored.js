// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.instant.from
description: The receiver is never called when calling from()
includes: [temporalHelpers.js]
features: [Temporal]
---*/

TemporalHelpers.checkSubclassingIgnoredStatic(
  Temporal.Instant,
  "from",
  ["1976-11-18T14:23:30.123456789Z"],
  (result) => assert.sameValue(result.epochNanoseconds, 217_175_010_123_456_789n, "epochNanoseconds result"),
);
