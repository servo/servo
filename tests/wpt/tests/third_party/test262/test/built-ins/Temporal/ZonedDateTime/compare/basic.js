// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.compare
description: Temporal.ZonedDateTime.compare works
features: [Temporal]
---*/

/*
const zdt1 = Temporal.ZonedDateTime.from("1976-11-18T15:23:30.123456789+01:00[+01:00]");
const zdt2 = Temporal.ZonedDateTime.from("2019-10-29T10:46:38.271986102+01:00[+01:00]");
*/
const zdt1 = new Temporal.ZonedDateTime(217175010123456789n, "+01:00");
const zdt2 = new Temporal.ZonedDateTime(1572342398271986102n, "+01:00");

// equal
assert.sameValue(Temporal.ZonedDateTime.compare(zdt1, zdt1), 0)

// smaller/larger
assert.sameValue(Temporal.ZonedDateTime.compare(zdt1, zdt2), -1)

// larger/smaller
assert.sameValue(Temporal.ZonedDateTime.compare(zdt2, zdt1), 1)
