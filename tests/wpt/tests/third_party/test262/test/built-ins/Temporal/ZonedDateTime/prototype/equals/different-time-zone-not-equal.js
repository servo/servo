// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.equals
description: Different time zones not equal.
features: [Temporal]
---*/

const zdt = new Temporal.ZonedDateTime(0n, "-05:00", "iso8601");
const zdt2 = new Temporal.ZonedDateTime(0n, "UTC", "iso8601");

assert(!zdt.equals(zdt2));

