// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth
description: Calendar ID is canonicalized
features: [Temporal]
---*/

var result = new Temporal.PlainYearMonth(2024, 6, "islamicc", 8);
assert.sameValue(result.calendarId, "islamic-civil", "calendar ID is canonicalized");

// May need to be removed in the future.
// See https://github.com/tc39/ecma402/issues/285
result = new Temporal.PlainYearMonth(2024, 6, "ethiopic-amete-alem");
assert.sameValue(result.calendarId, "ethioaa", "calendar ID is canonicalized");
