// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth
description: referenceISODay argument defaults to 1 if not given
features: [Temporal]
---*/

const args = [2000, 5];

const dateExplicit = new Temporal.PlainYearMonth(...args, undefined);
const isoDayExplicit = Number(dateExplicit.toString({ calendarName: "always" }).split("-")[2].slice(0, 2));
assert.sameValue(isoDayExplicit, 1, "default referenceISODay is 1");

const dateImplicit = new Temporal.PlainYearMonth(...args);
const isoDayImplicit = Number(dateImplicit.toString({ calendarName: "always" }).split("-")[2].slice(0, 2));
assert.sameValue(isoDayImplicit, 1, "default referenceISODay is 1");
