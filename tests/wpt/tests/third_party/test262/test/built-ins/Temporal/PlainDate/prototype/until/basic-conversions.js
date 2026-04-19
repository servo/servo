// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.prototype.until
description: Test that until() can take a string or property bag argument
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const plainDate = new Temporal.PlainDate(1969, 7, 24);

TemporalHelpers.assertDuration(plainDate.until({ year: 2019, month: 7, day: 24 }), 0, 0, 0, /* days = */ 18262, 0, 0, 0, 0, 0, 0, "option bag");
TemporalHelpers.assertDuration(plainDate.until("2019-07-24"), 0, 0, 0, /* days = */ 18262, 0, 0, 0, 0, 0, 0, "string");
