// Copyright (C) 2020 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.prototype.subtract
description: A string is parsed into the correct object when passed as the argument
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const instance = Temporal.PlainDate.from({ year: 2000, month: 5, day: 2 });
TemporalHelpers.assertPlainDate(instance.subtract("P3D"), 2000, 4, "M04", 29);
TemporalHelpers.assertPlainDate(instance.subtract("-P3D"), 2000, 5, "M05", 5);
