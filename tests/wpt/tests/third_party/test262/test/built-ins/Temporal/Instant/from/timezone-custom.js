// Copyright (C) 2020 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.instant.from
description: Time zone annotation is ignored in input ISO string
features: [Temporal]
---*/

const instant = Temporal.Instant.from("1975-02-02T14:25:36.123456789+01:00[Invalid/TimeZone]");
assert.sameValue(instant.epochMilliseconds, 160579536123);
