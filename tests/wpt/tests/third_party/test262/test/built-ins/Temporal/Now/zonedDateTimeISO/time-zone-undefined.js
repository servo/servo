// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.now.zoneddatetimeiso
description: Functions when time zone argument is omitted
features: [Temporal]
---*/

const systemTimeZone = Temporal.Now.timeZoneId();

const resultExplicit = Temporal.Now.zonedDateTimeISO(undefined);
assert.sameValue(resultExplicit.timeZoneId, systemTimeZone, "time zone string should be the system time zone");

const resultImplicit = Temporal.Now.zonedDateTimeISO();
assert.sameValue(resultImplicit.timeZoneId, systemTimeZone, "time zone string should be the system time zone");
