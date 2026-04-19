// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainmonthday
description: referenceISOYear argument defaults to 1972 if not given
features: [Temporal]
---*/

const args = [5, 2, "iso8601"];

const dateExplicit = new Temporal.PlainMonthDay(...args, undefined);
const isoYearExplicit = Number(dateExplicit.toString({ calendarName: "always" }).slice(0, 4));
assert.sameValue(isoYearExplicit, 1972, "default referenceISOYear is 1972");

const dateImplicit = new Temporal.PlainMonthDay(...args);
const isoYearImplicit = Number(dateImplicit.toString({ calendarName: "always" }).slice(0, 4));
assert.sameValue(isoYearImplicit, 1972, "default referenceISOYear is 1972");
