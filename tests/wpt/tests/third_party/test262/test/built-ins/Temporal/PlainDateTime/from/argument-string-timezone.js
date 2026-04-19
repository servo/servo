// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.from
description: Timezone, if specified, is ignored
features: [Temporal]
includes: [temporalHelpers.js]
---*/

TemporalHelpers.assertPlainDateTime(
  Temporal.PlainDateTime.from("2020-01-01T01:23:45[Asia/Kolkata]"),
  2020, 1, "M01", 1, 1, 23, 45, 0, 0, 0,
  "ignores if a timezone is specified"
);
