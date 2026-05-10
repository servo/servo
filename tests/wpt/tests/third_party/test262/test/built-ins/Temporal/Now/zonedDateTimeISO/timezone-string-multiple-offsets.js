// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.now.zoneddatetimeiso
description: Time zone parsing from ISO strings uses the bracketed offset, not the ISO string offset
features: [Temporal]
---*/

const timeZone = "2021-08-19T17:30:45.123456789-12:12[+01:46]";

const result = Temporal.Now.zonedDateTimeISO(timeZone);
assert.sameValue(result.timeZoneId, "+01:46", "Time zone string determined from bracket name");
