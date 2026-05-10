// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.compare
description: Compares exact time, not clock time.
features: [Temporal]
---*/

/*
const clockBefore = Temporal.ZonedDateTime.from("1999-12-31T23:30-08:00[-08:00]");
const clockAfter = Temporal.ZonedDateTime.from("2000-01-01T01:30-04:00[-04:00]");
*/
const clockBefore = new Temporal.ZonedDateTime(946711800000000000n, "-08:00");
const clockAfter = new Temporal.ZonedDateTime(946704600000000000n, "-04:00");

assert.sameValue(Temporal.ZonedDateTime.compare(clockBefore, clockAfter), 1);
assert.sameValue(Temporal.PlainDateTime.compare(clockBefore.toPlainDateTime(), clockAfter.toPlainDateTime()), -1);

