// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.until
description: until() casts its argument to a ZonedDateTime.
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const zdt = Temporal.ZonedDateTime.from("1976-11-18T15:23:30.123456789+01:00[+01:00]");

TemporalHelpers.assertDuration(
    zdt.until({
        year: 2019,
        month: 10,
        day: 29,
        hour: 10,
        timeZone: "+01:00"
    }), 0, 0, 0, 0, 376434, 36, 29, 876, 543, 211);

TemporalHelpers.assertDuration(
    zdt.until("2019-10-29T10:46:38.271986102+01:00[+01:00]"),
    0, 0, 0, 0, 376435, 23, 8, 148, 529, 313);

