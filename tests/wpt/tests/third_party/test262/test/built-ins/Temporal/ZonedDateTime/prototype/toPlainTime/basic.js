// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.toplaintime
description: toPlainTime() works as expected.
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const zdt = Temporal.ZonedDateTime.from("2019-10-29T09:46:38.271986102Z[-07:00]");

TemporalHelpers.assertPlainTime(zdt.toPlainTime(), 2, 46, 38, 271, 986, 102);
