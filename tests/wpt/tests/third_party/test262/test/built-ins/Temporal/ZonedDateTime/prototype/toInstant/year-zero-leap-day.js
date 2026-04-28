// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.toinstant
description: Year 0 leap day.
includes: [temporalHelpers.js]
features: [Temporal]
---*/

var zdt = Temporal.ZonedDateTime.from("0000-02-29T00:00-00:01[-00:01]");
TemporalHelpers.assertInstantsEqual(
    zdt.toInstant(),
    Temporal.Instant.from("0000-02-29T00:01:00Z"));

zdt = Temporal.ZonedDateTime.from("+000000-02-29T00:00-00:01[-00:01]");
TemporalHelpers.assertInstantsEqual(
    zdt.toInstant(),
    Temporal.Instant.from("0000-02-29T00:01:00Z"));
