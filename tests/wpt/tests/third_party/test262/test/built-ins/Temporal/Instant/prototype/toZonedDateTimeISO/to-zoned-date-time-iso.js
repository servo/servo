// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.instant.prototype.tozoneddatetimeiso
description: Test time zone parameters.
features: [Temporal]
---*/

const inst = new Temporal.Instant(1_000_000_000_000_000_000n);

// time zone parameter UTC
const zdt = inst.toZonedDateTimeISO("UTC");
assert.sameValue(inst.epochNanoseconds, zdt.epochNanoseconds);
assert.sameValue(zdt.timeZoneId, "UTC");

// time zone parameter non-UTC
const zdtNonUTC = inst.toZonedDateTimeISO("-05:00");
assert.sameValue(inst.epochNanoseconds, zdtNonUTC.epochNanoseconds);
assert.sameValue(zdtNonUTC.timeZoneId, "-05:00");
