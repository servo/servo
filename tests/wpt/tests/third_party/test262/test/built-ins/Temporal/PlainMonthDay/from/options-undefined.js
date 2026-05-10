// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainmonthday.from
includes: [temporalHelpers.js]
description: Verify that undefined options are handled correctly.
features: [Temporal]
---*/

const fields = { month: 2, day: 31 };

const explicit = Temporal.PlainMonthDay.from(fields, undefined);
TemporalHelpers.assertPlainMonthDay(explicit, "M02", 29, "default overflow is constrain");

const implicit = Temporal.PlainMonthDay.from(fields);
TemporalHelpers.assertPlainMonthDay(implicit, "M02", 29, "default overflow is constrain");

const lambda = Temporal.PlainMonthDay.from(fields, () => {});
TemporalHelpers.assertPlainMonthDay(lambda, "M02", 29, "default overflow is constrain");
