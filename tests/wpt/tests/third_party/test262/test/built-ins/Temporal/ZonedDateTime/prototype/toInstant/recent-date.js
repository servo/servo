// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.toinstant
description: toInstant() works with a recent date.
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const zdt = Temporal.ZonedDateTime.from("2019-10-29T10:46:38.271986102+01:00[+01:00]");

TemporalHelpers.assertInstantsEqual(
    zdt.toInstant(),
    Temporal.Instant.from("2019-10-29T09:46:38.271986102Z"));
