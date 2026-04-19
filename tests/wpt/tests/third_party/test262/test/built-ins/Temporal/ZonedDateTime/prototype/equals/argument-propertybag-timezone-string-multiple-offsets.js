// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.equals
description: Time zone parsing from ISO strings uses the bracketed offset, not the ISO string offset
features: [Temporal]
---*/

const expectedTimeZone = "+01:46";
const instance = new Temporal.ZonedDateTime(0n, expectedTimeZone);
const timeZone = "2021-08-19T17:30:45.123456789-12:12[+01:46]";

// This operation should produce expectedTimeZone, so the following should
// be equal due to the time zones being different on the receiver and
// the argument.

const properties = { year: 1970, month: 1, day: 1, hour: 1, minute: 46 };
assert(instance.equals({ ...properties, timeZone }), "time zone string should produce expected time zone");
