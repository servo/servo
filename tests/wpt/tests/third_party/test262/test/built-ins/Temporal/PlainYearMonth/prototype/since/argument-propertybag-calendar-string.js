// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.prototype.since
description: A calendar ID is valid input for Calendar
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const instance = new Temporal.PlainYearMonth(2019, 6);

const calendar = "iso8601";

const arg = { year: 2019, monthCode: "M06", calendar };
const result = instance.since(arg);
TemporalHelpers.assertDuration(result, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, `Calendar created from string "${calendar}"`);
