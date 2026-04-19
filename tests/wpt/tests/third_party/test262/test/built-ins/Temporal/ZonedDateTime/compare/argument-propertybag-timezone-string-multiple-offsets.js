// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.compare
description: Time zone parsing from ISO strings uses the bracketed offset, not the ISO string offset
features: [Temporal]
---*/

const instance = new Temporal.ZonedDateTime(1588371240_000_000_000n, "+01:46");

const timeZone = "2021-08-19T17:30:45.123456789-12:12[+01:46]";

const result1 = Temporal.ZonedDateTime.compare({ year: 2020, month: 5, day: 2, timeZone }, instance);
assert.sameValue(result1, 0, "Time zone string determined from bracket name (first argument)");
const result2 = Temporal.ZonedDateTime.compare(instance, { year: 2020, month: 5, day: 2, timeZone });
assert.sameValue(result2, 0, "Time zone string determined from bracket name (second argument)");
