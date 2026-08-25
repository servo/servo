// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.prototype.since
description: since() should take length of month into account.
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const plainDate1 = Temporal.PlainDate.from("2019-01-01");
const plainDate2 = Temporal.PlainDate.from("2019-02-01");
const plainDate3 = Temporal.PlainDate.from("2019-03-01");
TemporalHelpers.assertDuration(plainDate2.since(plainDate1), 0, 0, 0, /* days = */ 31, 0, 0, 0, 0, 0, 0, "January 2019");
TemporalHelpers.assertDuration(plainDate3.since(plainDate2), 0, 0, 0, /* days = */ 28, 0, 0, 0, 0, 0, 0, "February 2019");

const plainDate4 = Temporal.PlainDate.from("2020-02-01");
const plainDate5 = Temporal.PlainDate.from("2020-03-01");
TemporalHelpers.assertDuration(plainDate5.since(plainDate4), 0, 0, 0, /* days = */ 29, 0, 0, 0, 0, 0, 0, "February 2020");
