// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.compare
description: Disregards time zone IDs if exact times are equal.
features: [Temporal]
---*/

/*
const zdt1 = Temporal.ZonedDateTime.from("1976-11-18T15:23:30.123456789+01:00[+01:00]");
*/
const zdt1 = new Temporal.ZonedDateTime(217175010123456789n, "+01:00");

assert.sameValue(Temporal.ZonedDateTime.compare(zdt1, zdt1.withTimeZone("+05:30")), 0);
