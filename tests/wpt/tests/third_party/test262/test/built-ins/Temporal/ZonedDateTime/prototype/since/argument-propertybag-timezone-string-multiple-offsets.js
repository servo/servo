// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.since
description: Time zone parsing from ISO strings uses the bracketed offset, not the ISO string offset
features: [Temporal]
---*/

const expectedTimeZone = "+01:46";
const instance = new Temporal.ZonedDateTime(0n, expectedTimeZone);
const timeZone = "2021-08-19T17:30:45.123456789-12:12[+01:46]";

// This operation should produce expectedTimeZone, so the following operation
// should not throw due to the time zones being different on the receiver and
// the argument.

instance.since({ year: 2020, month: 5, day: 2, timeZone });
