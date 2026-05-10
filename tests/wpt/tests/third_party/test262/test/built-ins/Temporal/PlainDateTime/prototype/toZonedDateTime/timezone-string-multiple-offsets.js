// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.tozoneddatetime
description: Time zone parsing from ISO strings uses the bracketed offset, not the ISO string offset
features: [Temporal]
---*/

const instance = new Temporal.PlainDateTime(2000, 5, 2);
const timeZone = "2021-08-19T17:30:45.123456789-12:12[+01:46]";

const result = instance.toZonedDateTime(timeZone);
assert.sameValue(result.timeZoneId, "+01:46", "Time zone string determined from bracket name");
