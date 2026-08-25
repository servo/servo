// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.prototype.equals
description: A calendar ID is valid input for Calendar
features: [Temporal]
---*/

const instance = new Temporal.PlainYearMonth(2019, 6);

const calendar = "iso8601";

const arg = { year: 2019, monthCode: "M06", calendar };
const result = instance.equals(arg);
assert.sameValue(result, true, `Calendar created from string "${calendar}"`);
