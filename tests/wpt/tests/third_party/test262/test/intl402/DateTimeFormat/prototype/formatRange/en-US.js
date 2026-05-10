// Copyright (C) 2019 the V8 project authors, Igalia S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-partitiondatetimerangepattern
description: Basic tests for the en-US output of formatRange()
info: |
  Intl.DateTimeFormat.prototype.formatRange ( startDate , endDate )

  8. Return ? FormatDateTimeRange(dtf, x, y).
locale: [en-US]
features: [Intl.DateTimeFormat-formatRange]
---*/

// Tolerate implementation variance by expecting consistency without being prescriptive.
// TODO: can we change tests to be less reliant on CLDR formats while still testing that
// Temporal and Intl are behaving as expected?
const usDateRangeSeparator = new Intl.DateTimeFormat("en-US", { dateStyle: "short" })
  .formatRangeToParts(1 * 86400 * 1000, 366 * 86400 * 1000)
  .find((part) => part.type === "literal" && part.source === "shared").value;

const date1 = new Date("2019-01-03T00:00:00");
const date2 = new Date("2019-01-05T00:00:00");
const date3 = new Date("2019-03-04T00:00:00");
const date4 = new Date("2020-03-04T00:00:00");

let dtf = new Intl.DateTimeFormat("en-US");
assert.sameValue(dtf.formatRange(date1, date1), "1/3/2019");
assert.sameValue(dtf.formatRange(date1, date2), `1/3/2019${usDateRangeSeparator}1/5/2019`);
assert.sameValue(dtf.formatRange(date1, date3), `1/3/2019${usDateRangeSeparator}3/4/2019`);
assert.sameValue(dtf.formatRange(date1, date4), `1/3/2019${usDateRangeSeparator}3/4/2020`);
assert.sameValue(dtf.formatRange(date2, date3), `1/5/2019${usDateRangeSeparator}3/4/2019`);
assert.sameValue(dtf.formatRange(date2, date4), `1/5/2019${usDateRangeSeparator}3/4/2020`);
assert.sameValue(dtf.formatRange(date3, date4), `3/4/2019${usDateRangeSeparator}3/4/2020`);

dtf = new Intl.DateTimeFormat("en-US", {year: "numeric", month: "short", day: "numeric"});
assert.sameValue(dtf.formatRange(date1, date1), "Jan 3, 2019");
assert.sameValue(dtf.formatRange(date1, date2), `Jan 3${usDateRangeSeparator}5, 2019`);
assert.sameValue(dtf.formatRange(date1, date3), `Jan 3${usDateRangeSeparator}Mar 4, 2019`);
assert.sameValue(dtf.formatRange(date1, date4), `Jan 3, 2019${usDateRangeSeparator}Mar 4, 2020`);
assert.sameValue(dtf.formatRange(date2, date3), `Jan 5${usDateRangeSeparator}Mar 4, 2019`);
assert.sameValue(dtf.formatRange(date2, date4), `Jan 5, 2019${usDateRangeSeparator}Mar 4, 2020`);
assert.sameValue(dtf.formatRange(date3, date4), `Mar 4, 2019${usDateRangeSeparator}Mar 4, 2020`);
