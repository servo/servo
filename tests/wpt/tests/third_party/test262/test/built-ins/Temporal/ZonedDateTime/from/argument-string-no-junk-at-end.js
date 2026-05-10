// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.from
description: No junk at end of string.
features: [Temporal]
---*/

assert.throws(RangeError, () => Temporal.ZonedDateTime.from("1976-11-18T15:23:30.123456789-08:00[-08:00]junk"))

