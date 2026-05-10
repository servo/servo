// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainmonthday
description: Calendar ID is canonicalized
features: [Temporal]
---*/

var result = new Temporal.PlainMonthDay(2, 11, "islamicc", 1972);
assert.sameValue(result.calendarId, "islamic-civil", "calendar ID is canonicalized");

// May need to be removed in the future.
// See https://github.com/tc39/ecma402/issues/285
result = new Temporal.PlainMonthDay(2, 11, "ethiopic-amete-alem");
assert.sameValue(result.calendarId, "ethioaa", "calendar ID is canonicalized");
