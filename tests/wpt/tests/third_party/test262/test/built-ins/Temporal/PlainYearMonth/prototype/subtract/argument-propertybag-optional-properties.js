// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.prototype.subtract
description: >
  A duration property bag with value 0 for disallowed properties is the same as
  a property bag with no disallowed properties
features: [Temporal]
---*/

const instance = new Temporal.PlainYearMonth(1970, 1);

const oneProperty = {
  months: 1,
};
const allProperties = {
  years: 0,
  months: 1,
  weeks: 0,
  days: 0,
  hours: 0,
  minutes: 0,
  seconds: 0,
  milliseconds: 0,
  microseconds: 0,
  nanoseconds: 0,
};
const resultWithout = instance.subtract(oneProperty);
const resultWith = instance.subtract(allProperties);
assert(resultWithout.equals(resultWith), "results should be the same with 0 for disallowed properties and without disallowed properties");
