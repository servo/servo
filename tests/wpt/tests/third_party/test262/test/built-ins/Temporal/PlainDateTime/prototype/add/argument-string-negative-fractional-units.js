// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.add
description: Strings with fractional duration units are treated with the correct sign
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const instance = new Temporal.PlainDateTime(2000, 5, 2);

const resultHours = instance.add("-PT24.567890123H");
TemporalHelpers.assertPlainDateTime(resultHours, 2000, 4, "M04", 30, 23, 25, 55, 595, 557, 200, "negative fractional hours");

const resultMinutes = instance.add("-PT1440.567890123M");
TemporalHelpers.assertPlainDateTime(resultMinutes, 2000, 4, "M04", 30, 23, 59, 25, 926, 592, 620, "negative fractional minutes");
