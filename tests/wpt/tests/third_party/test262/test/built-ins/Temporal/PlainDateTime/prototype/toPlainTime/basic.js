// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.toplaintime
description: Basic usage
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const plainDateTime = Temporal.PlainDateTime.from("2020-02-12T11:42:56.987654321+01:00[Europe/Amsterdam]");
TemporalHelpers.assertPlainTime(plainDateTime.toPlainTime(), 11, 42, 56, 987, 654, 321);
